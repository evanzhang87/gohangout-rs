//! File watcher tests

use gohangout_rs::watcher::ConfigWatcher;
use tempfile::NamedTempFile;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn test_watcher_detects_file_change() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_path_buf();
    
    // Write initial content
    std::fs::write(&file_path, "initial content").expect("Failed to write file");
    
    // Create watcher
    let mut watcher = ConfigWatcher::new(&file_path, Duration::from_millis(100))
        .await
        .expect("Failed to create watcher");
    
    // Wait a bit for watcher to be ready
    time::sleep(Duration::from_millis(200)).await;
    
    // Modify file
    std::fs::write(&file_path, "modified content").expect("Failed to modify file");
    
    // Should detect change within timeout
    let changed = time::timeout(Duration::from_secs(2), watcher.wait_for_change())
        .await
        .expect("Timeout waiting for change");
    
    assert!(changed.is_ok());
}

#[tokio::test]
async fn test_watcher_debounce() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_path_buf();
    
    // Write initial content
    std::fs::write(&file_path, "content").expect("Failed to write file");
    
    // Create watcher with debounce
    let mut watcher = ConfigWatcher::new(&file_path, Duration::from_millis(500))
        .await
        .expect("Failed to create watcher");
    
    // Wait a bit
    time::sleep(Duration::from_millis(200)).await;
    
    // Rapid modifications
    for i in 0..5 {
        std::fs::write(&file_path, format!("content {}", i)).expect("Failed to modify file");
        time::sleep(Duration::from_millis(50)).await;
    }
    
    // Should only get one notification due to debounce
    let start = std::time::Instant::now();
    let changed = time::timeout(Duration::from_secs(2), watcher.wait_for_change())
        .await
        .expect("Timeout waiting for change");
    
    assert!(changed.is_ok());
    
    // Should have waited for debounce period
    assert!(start.elapsed() >= Duration::from_millis(500));
}

#[tokio::test]
async fn test_watcher_nonexistent_file() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_path_buf();
    
    // Delete file to make it nonexistent
    std::fs::remove_file(&file_path).expect("Failed to delete file");
    
    // Should fail to create watcher for nonexistent file
    let result = ConfigWatcher::new(&file_path, Duration::from_millis(100)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_watcher_stop() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_path_buf();
    
    std::fs::write(&file_path, "content").expect("Failed to write file");
    
    let mut watcher = ConfigWatcher::new(&file_path, Duration::from_millis(100))
        .await
        .expect("Failed to create watcher");
    
    // Stop the watcher
    watcher.stop().await.expect("Failed to stop watcher");
    
    // Modify file after stopping
    std::fs::write(&file_path, "modified").expect("Failed to modify file");
    
    // Wait a bit
    time::sleep(Duration::from_millis(200)).await;
    
    // Should not detect change since watcher is stopped
    let result = time::timeout(Duration::from_millis(300), watcher.wait_for_change()).await;
    assert!(result.is_err()); // Should timeout
}

#[tokio::test]
async fn test_watcher_directory_watch() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let config_file = temp_dir.path().join("config.yaml");
    
    // Create initial file
    std::fs::write(&config_file, "initial").expect("Failed to write file");
    
    let mut watcher = ConfigWatcher::new(&config_file, Duration::from_millis(100))
        .await
        .expect("Failed to create watcher");
    
    time::sleep(Duration::from_millis(200)).await;
    
    // Create a new file in the directory (not the watched file)
    let other_file = temp_dir.path().join("other.yaml");
    std::fs::write(&other_file, "other content").expect("Failed to write other file");
    
    // Should not trigger for different file
    let result = time::timeout(Duration::from_millis(300), watcher.wait_for_change()).await;
    assert!(result.is_err()); // Should timeout
    
    // Now modify the watched file
    std::fs::write(&config_file, "modified").expect("Failed to modify file");
    
    // Should detect change
    let changed = time::timeout(Duration::from_secs(2), watcher.wait_for_change())
        .await
        .expect("Timeout waiting for change");
    
    assert!(changed.is_ok());
}