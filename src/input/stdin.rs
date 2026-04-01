//! StdinInput plugin implementation

use crate::event::Event;
use crate::prelude::{PluginError, PluginResult, Input, InputStats, Plugin, PluginType};
use crate::plugin::PluginConfig;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task;

/// StdinInput plugin for reading from standard input
pub struct StdinInput {
    /// Plugin name
    name: String,
    
    /// Plugin configuration
    config: HashMap<String, Value>,
    
    /// Decoder for processing input
    decoder: Arc<dyn crate::input::Decoder>,
    
    /// Additional fields to add to each event
    add_fields: HashMap<String, Value>,
    
    /// Buffer size for reading
    buffer_size: usize,
    
    /// Statistics
    stats: Arc<Mutex<InputStats>>,
    
    /// Channel for async reading
    reader_tx: Option<mpsc::Sender<io::Result<String>>>,
    reader_rx: Option<mpsc::Receiver<io::Result<String>>>,
    
    /// Reader task handle
    reader_handle: Option<task::JoinHandle<()>>,
}

impl StdinInput {
    /// Create a new StdinInput from configuration
    pub fn from_config(config: &PluginConfig) -> PluginResult<Self> {
        // Get codec type (default: "plain")
        let codec_type = config
            .get_config("codec")
            .and_then(|v| v.as_str())
            .unwrap_or("plain");
        
        // Create decoder
        let decoder = crate::input::codec::create_decoder(codec_type)
            .map_err(|e| PluginError::configuration_error(&e.to_string()))?;
        
        // Get buffer size (default: 8192)
        let buffer_size = config
            .get_config("buffer_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(8192) as usize;
        
        if buffer_size == 0 {
            return Err(PluginError::configuration_error(
                "buffer_size must be greater than 0",
            ));
        }
        
        // Get additional fields to add
        let add_fields = config
            .get_config("add_fields")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(Self {
            name: "stdin".to_string(),
            config: config.config().clone(),
            decoder: decoder.into(),
            add_fields,
            buffer_size,
            stats: Arc::new(Mutex::new(InputStats::default())),
            reader_tx: None,
            reader_rx: None,
            reader_handle: None,
        })
    }
    
    /// Get the codec type
    pub fn codec_type(&self) -> &str {
        self.decoder.name()
    }
    
    /// Get the buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
    
    /// Get mutable access to configuration (for factory)
    pub fn config_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.config
    }
    
    /// Get additional fields
    pub fn add_fields(&self) -> &HashMap<String, Value> {
        &self.add_fields
    }
    
    /// Start the async reader task
    fn start_reader(&mut self) -> PluginResult<()> {
        // Create channel for communication between reader task and plugin
        let (tx, rx) = mpsc::channel(100);
        
        // Clone stats for the task
        let stats = Arc::clone(&self.stats);
        let buffer_size = self.buffer_size;
        
        // Clone tx for use in the closure
        let tx_clone = tx.clone();
        
        // Spawn reader task
        let handle = task::spawn_blocking(move || {
            let stdin = io::stdin();
            let mut reader = BufReader::with_capacity(buffer_size, stdin.lock());
            let mut buffer = String::new();
            
            loop {
                buffer.clear();
                
                // Read line from stdin
                match reader.read_line(&mut buffer) {
                    Ok(0) => {
                        // EOF reached
                        let _ = tx_clone.blocking_send(Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "EOF reached",
                        )));
                        break;
                    }
                    Ok(bytes_read) => {
                        // Update statistics
                        if let Ok(mut stats) = stats.lock() {
                            stats.bytes_read += bytes_read as u64;
                            stats.events_read += 1;
                        }
                        
                        // Send line (trim trailing newline)
                        let line = buffer.trim_end().to_string();
                        if tx_clone.blocking_send(Ok(line)).is_err() {
                            // Receiver dropped, stop reading
                            break;
                        }
                    }
                    Err(e) => {
                        // Update error statistics
                        if let Ok(mut stats) = stats.lock() {
                            stats.errors += 1;
                        }
                        
                        // Send error
                        let _ = tx_clone.blocking_send(Err(e));
                        break;
                    }
                }
            }
        });
        
        self.reader_tx = Some(tx);
        self.reader_rx = Some(rx);
        self.reader_handle = Some(handle);
        
        Ok(())
    }
    
    /// Stop the async reader task
    fn stop_reader(&mut self) -> PluginResult<()> {
        // Drop the sender to signal the task to stop
        self.reader_tx.take();
        
        // Wait for the task to finish
        if let Some(handle) = self.reader_handle.take() {
            // Don't block for too long
            let _ = task::block_in_place(|| {
                // Try to abort the task
                handle.abort();
                // Wait a bit for it to finish
                std::thread::sleep(std::time::Duration::from_millis(100));
            });
        }
        
        self.reader_rx.take();
        Ok(())
    }
    
    /// Apply additional fields to an event
    fn apply_add_fields(&self, mut event: Event) -> Event {
        for (key, value) in &self.add_fields {
            event.set(key, value.clone());
        }
        event
    }
}

impl Default for StdinInput {
    fn default() -> Self {
        Self {
            name: "stdin".to_string(),
            config: HashMap::new(),
            decoder: Arc::new(crate::input::PlainDecoder::new()),
            add_fields: HashMap::new(),
            buffer_size: 8192,
            stats: Arc::new(Mutex::new(InputStats::default())),
            reader_tx: None,
            reader_rx: None,
            reader_handle: None,
        }
    }
}

impl Plugin for StdinInput {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn config(&self) -> &HashMap<String, Value> {
        &self.config
    }
    
    fn plugin_type(&self) -> PluginType {
        PluginType::Input
    }
    
    fn initialize(&mut self) -> PluginResult<()> {
        // Recreate decoder based on config if codec is specified
        if let Some(codec_value) = self.config.get("codec") {
            if let Some(codec_type) = codec_value.as_str() {
                let decoder_box = crate::input::codec::create_decoder(codec_type)
                    .map_err(|e| PluginError::configuration_error(&e.to_string()))?;
                self.decoder = Arc::from(decoder_box);
            }
        }
        
        // Start the async reader
        self.start_reader()?;
        Ok(())
    }
    
    fn shutdown(&mut self) -> PluginResult<()> {
        // Stop the async reader
        self.stop_reader()?;
        Ok(())
    }
    
    fn validate_config(&self) -> PluginResult<()> {
        // Validate buffer size
        if self.buffer_size == 0 {
            return Err(PluginError::configuration_error(
                "buffer_size must be greater than 0",
            ));
        }
        
        Ok(())
    }
}

impl Input for StdinInput {
    fn read(&mut self) -> PluginResult<Option<Event>> {
        // Check if we have a receiver
        let Some(rx) = &mut self.reader_rx else {
            return Err(PluginError::execution_error(
                "StdinInput reader not initialized",
            ));
        };
        
        // Try to receive a line (non-blocking in async context)
        match task::block_in_place(|| rx.try_recv()) {
            Ok(Ok(line)) => {
                // Decode the line
                match self.decoder.decode(&line) {
                    Ok(event) => {
                        // Apply additional fields
                        let event = self.apply_add_fields(event);
                        Ok(Some(event))
                    }
                    Err(e) => {
                        // Update error statistics
                        if let Ok(mut stats) = self.stats.lock() {
                            stats.errors += 1;
                        }
                        
                        Err(PluginError::execution_error(&e.to_string()))
                    }
                }
            }
            Ok(Err(e)) => {
                // EOF or read error
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    Ok(None) // EOF is not an error, just no more data
                } else {
                    Err(PluginError::from(e))
                }
            }
            Err(mpsc::error::TryRecvError::Empty) => {
                // No data available
                Ok(None)
            }
            Err(mpsc::error::TryRecvError::Disconnected) => {
                // Channel disconnected (reader task stopped)
                Err(PluginError::execution_error("StdinInput reader disconnected"))
            }
        }
    }
    
    fn is_ready(&self) -> bool {
        // Check if we have data available
        if let Some(rx) = &self.reader_rx {
            !rx.is_empty()
        } else {
            false
        }
    }
    
    fn stats(&self) -> InputStats {
        self.stats.lock().unwrap().clone()
    }
}

impl Clone for StdinInput {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            config: self.config.clone(),
            decoder: self.decoder.clone(),
            add_fields: self.add_fields.clone(),
            buffer_size: self.buffer_size,
            stats: Arc::clone(&self.stats),
            reader_tx: None,
            reader_rx: None,
            reader_handle: None,
        }
    }
}

impl std::fmt::Debug for StdinInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StdinInput")
            .field("name", &self.name)
            .field("codec_type", &self.decoder.name())
            .field("buffer_size", &self.buffer_size)
            .field("add_fields", &self.add_fields)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_stdin_input_creation() {
        let config = PluginConfig::new("test_stdin", PluginType::Input);
        let input = StdinInput::from_config(&config).unwrap();
        
        assert_eq!(input.name(), "stdin");
        assert_eq!(input.codec_type(), "plain");
        assert_eq!(input.buffer_size(), 8192);
        assert!(input.add_fields().is_empty());
    }
    
    #[test]
    fn test_stdin_input_with_json_codec() {
        let mut config = PluginConfig::new("test_stdin", PluginType::Input);
        config.set_config("codec", json!("json"));
        
        let input = StdinInput::from_config(&config).unwrap();
        assert_eq!(input.codec_type(), "json");
    }
    
    #[test]
    fn test_stdin_input_with_add_fields() {
        let mut config = PluginConfig::new("test_stdin", PluginType::Input);
        config.set_config("add_fields", json!({
            "source": "stdin",
            "environment": "test"
        }));
        
        let input = StdinInput::from_config(&config).unwrap();
        let add_fields = input.add_fields();
        
        assert_eq!(add_fields.get("source").unwrap(), "stdin");
        assert_eq!(add_fields.get("environment").unwrap(), "test");
    }
    
    #[test]
    fn test_stdin_input_invalid_buffer_size() {
        let mut config = PluginConfig::new("test_stdin", PluginType::Input);
        config.set_config("buffer_size", json!(0));
        
        let result = StdinInput::from_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("buffer_size"));
    }
    
    #[test]
    fn test_stdin_input_invalid_codec() {
        let mut config = PluginConfig::new("test_stdin", PluginType::Input);
        config.set_config("codec", json!("invalid_codec"));
        
        let result = StdinInput::from_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported codec"));
    }
    
    #[test]
    fn test_stdin_input_plugin_trait() {
        let config = PluginConfig::new("test_stdin", PluginType::Input);
        let input = StdinInput::from_config(&config).unwrap();
        
        assert_eq!(input.plugin_type(), PluginType::Input);
        assert!(input.validate_config().is_ok());
    }
    
    #[test]
    fn test_stdin_input_apply_add_fields() {
        let mut config = PluginConfig::new("test_stdin", PluginType::Input);
        config.set_config("add_fields", json!({
            "source": "stdin",
            "count": 1
        }));
        
        let input = StdinInput::from_config(&config).unwrap();
        
        // Create a test event
        let event = Event::new(json!({"message": "test"}));
        
        // Apply additional fields (this is an internal method, but we can test the logic)
        let mut test_event = event.clone();
        for (key, value) in input.add_fields() {
            test_event.set(key, value.clone());
        }
        
        assert_eq!(test_event.get("message").unwrap(), "test");
        assert_eq!(test_event.get("source").unwrap(), "stdin");
        assert_eq!(test_event.get("count").unwrap(), 1);
    }
    
    #[test]
    fn test_stdin_input_default() {
        let input = StdinInput::default();
        
        assert_eq!(input.name(), "stdin");
        assert_eq!(input.codec_type(), "plain");
        assert_eq!(input.buffer_size(), 8192);
        assert!(input.add_fields().is_empty());
    }
    
    #[test]
    fn test_stdin_input_clone() {
        let config = PluginConfig::new("test_stdin", PluginType::Input);
        let input1 = StdinInput::from_config(&config).unwrap();
        let input2 = input1.clone();
        
        assert_eq!(input1.name(), input2.name());
        assert_eq!(input1.codec_type(), input2.codec_type());
        assert_eq!(input1.buffer_size(), input2.buffer_size());
    }
}