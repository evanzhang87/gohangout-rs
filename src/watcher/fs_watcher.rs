//! Simple file system watcher implementation

use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info};

/// Errors that can occur in the file watcher
#[derive(Debug, thiserror::Error)]
pub enum WatcherError {
    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    /// File system error
    #[error("File system error: {0}")]
    FileSystemError(#[source] std::io::Error),
    
    /// Notify library error
    #[error("File watcher error: {0}")]
    NotifyError(#[source] notify::Error),
    
    /// Timeout waiting for file change
    #[error("Timeout waiting for file change")]
    Timeout,
    
    /// Watcher has been stopped
    #[error("Watcher has been stopped")]
    Stopped,
}

/// Configuration file watcher
pub struct ConfigWatcher {
    /// Path to the watched file
    file_path: PathBuf,
    
    /// Debounce duration
    debounce_duration: Duration,
    
    /// Receiver for change events
    change_receiver: mpsc::Receiver<()>,
    
    /// Handle to the watcher task
    _watcher_handle: tokio::task::JoinHandle<()>,
}

impl ConfigWatcher {
    /// Create a new configuration file watcher
    pub async fn new<P: AsRef<Path>>(
        file_path: P,
        debounce_duration: Duration,
    ) -> std::result::Result<Self, WatcherError> {
        let file_path = file_path.as_ref().to_path_buf();
        
        // Check if file exists
        if !file_path.exists() {
            return Err(WatcherError::FileNotFound(file_path.clone()));
        }
        
        // Create channel for change events
        let (change_sender, change_receiver) = mpsc::channel(10);
        
        // Clone values for the async task
        let file_path_clone = file_path.clone();
        let debounce_duration_clone = debounce_duration;
        
        // Spawn the watcher task
        let watcher_handle = tokio::spawn(async move {
            if let Err(e) = Self::watch_file(
                &file_path_clone,
                debounce_duration_clone,
                change_sender,
            ).await {
                error!("File watcher error: {}", e);
            }
        });
        
        info!("Watching configuration file: {}", file_path.display());
        
        Ok(Self {
            file_path,
            debounce_duration,
            change_receiver,
            _watcher_handle: watcher_handle,
        })
    }
    
    /// Internal file watching logic
    async fn watch_file(
        file_path: &Path,
        debounce_duration: Duration,
        change_sender: mpsc::Sender<()>,
    ) -> std::result::Result<(), notify::Error> {
        let (event_sender, mut event_receiver) = mpsc::channel(100);
        
        // Create the file watcher
        let mut watcher = notify::recommended_watcher(move |res| {
            let sender = event_sender.clone();
            if let Ok(event) = res {
                let _ = sender.blocking_send(event);
            }
        })?;
        
        // Watch the parent directory
        let parent_dir = file_path.parent().ok_or_else(|| {
            notify::Error::generic("File has no parent directory")
        })?;
        
        watcher.watch(parent_dir, RecursiveMode::NonRecursive)?;
        
        let file_path = file_path.to_path_buf();
        let mut last_notification = None;
        
        loop {
            match event_receiver.recv().await {
                Some(event) => {
                    // Check if this event is relevant to our file
                    if Self::is_relevant_event(&file_path, &event) {
                        debug!("Detected relevant file change: {:?}", event.kind);
                        
                        let now = std::time::Instant::now();
                        let should_notify = match last_notification {
                            Some(last) => now.duration_since(last) >= debounce_duration,
                            None => true,
                        };
                        
                        if should_notify {
                            // Send notification after debounce
                            sleep(debounce_duration).await;
                            
                            // Check if we should still send (no newer events during debounce)
                            let _ = change_sender.try_send(());
                            last_notification = Some(std::time::Instant::now());
                        }
                    }
                }
                None => break,
            }
        }
        
        Ok(())
    }
    
    /// Wait for the configuration file to change
    pub async fn wait_for_change(&mut self) -> std::result::Result<(), WatcherError> {
        debug!("Waiting for configuration file change...");
        
        match timeout(Duration::from_secs(30), self.change_receiver.recv()).await {
            Ok(Some(())) => {
                info!("Configuration file changed, ready for reload");
                Ok(())
            }
            Ok(None) => Err(WatcherError::Stopped),
            Err(_) => Err(WatcherError::Timeout),
        }
    }
    
    /// Get the path being watched
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }
    
    /// Get the debounce duration
    pub fn debounce_duration(&self) -> Duration {
        self.debounce_duration
    }
    
    /// Check if an event is relevant to the watched file
    fn is_relevant_event(file_path: &Path, event: &Event) -> bool {
        // Check if our file is in the event paths
        event.paths.iter().any(|path| {
            // Direct match
            if path == file_path {
                return true;
            }
            
            // Handle rename events
            match event.kind {
                EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::From)) => {
                    // File was renamed from this path
                    path == file_path
                }
                EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::To)) => {
                    // File was renamed to this path
                    path.file_name() == file_path.file_name()
                }
                _ => false,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_watcher_creation() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        
        let watcher = ConfigWatcher::new(temp_file.path(), Duration::from_millis(100))
            .await
            .expect("Failed to create watcher");
        
        assert_eq!(watcher.file_path(), temp_file.path());
        assert_eq!(watcher.debounce_duration(), Duration::from_millis(100));
    }
}