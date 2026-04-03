//! StdoutOutput plugin implementation

use crate::event::Event;
use crate::prelude::{PluginError, PluginResult, Output, OutputStats, Plugin, PluginType};
use crate::plugin::PluginConfig;
use crate::plugin::traits::PluginStatus;
use crate::output::format::{OutputFormat, create_formatter, Formatter};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// StdoutOutput plugin for writing to standard output
pub struct StdoutOutput {
    /// Plugin name
    name: String,
    
    /// Plugin configuration
    config: HashMap<String, Value>,
    
    /// Output formatter
    formatter: Arc<dyn Formatter>,
    
    /// Output format
    format: OutputFormat,
    
    /// Buffer size for batch writing
    buffer_size: usize,
    
    /// Statistics
    stats: Arc<Mutex<OutputStats>>,
    
    /// Start time for statistics (protected by Mutex for interior mutability)
    start_time: Arc<Mutex<Instant>>,
    
    /// Last write time (protected by Mutex for interior mutability)
    last_write: Arc<Mutex<Option<Instant>>>,
    
    /// Write buffer for batch operations
    write_buffer: Arc<Mutex<Vec<Event>>>,
    
    /// Flush interval (milliseconds)
    flush_interval: u64,
    
    /// Last flush time (protected by Mutex for interior mutability)
    last_flush: Arc<Mutex<Option<Instant>>>,
}

impl StdoutOutput {
    /// Create a new StdoutOutput from configuration
    pub fn from_config(config: &PluginConfig) -> PluginResult<Self> {
        // Get plugin name
        let name = config.name.clone();
        
        // Get output format (default: json)
        let format_str = config
            .get_config("format")
            .and_then(|v| v.as_str())
            .unwrap_or("json");
        
        let format = OutputFormat::from_str(format_str)
            .ok_or_else(|| PluginError::ConfigurationError(
                format!("Invalid output format: {}", format_str)
            ))?;
        
        // Get buffer size (default: 8192)
        let buffer_size = config
            .get_config("buffer_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(8192) as usize;
        
        // Get flush interval (default: 1000ms)
        let flush_interval = config
            .get_config("flush_interval")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000);
        
        // Create formatter
        let config_map = config.config.clone();
        let formatter = create_formatter(format, &config_map)
            .map_err(|e| PluginError::ConfigurationError(
                format!("Failed to create formatter: {}", e)
            ))?;
        
        // Extract config map
        let config_map = config.config.clone();
        
        Ok(Self {
            name,
            config: config_map,
            formatter: Arc::from(formatter),
            format,
            buffer_size,
            stats: Arc::new(Mutex::new(OutputStats::default())),
            start_time: Arc::new(Mutex::new(Instant::now())),
            last_write: Arc::new(Mutex::new(None)),
            write_buffer: Arc::new(Mutex::new(Vec::with_capacity(buffer_size))),
            flush_interval,
            last_flush: Arc::new(Mutex::new(None)),
        })
    }
    
    /// Write formatted output to stdout
    fn write_formatted(&self, formatted: &str) -> PluginResult<()> {
        let mut stdout = io::stdout();
        
        // Write the formatted output
        stdout.write_all(formatted.as_bytes())
            .map_err(|e| PluginError::ExecutionError(
                format!("Failed to write to stdout: {}", e)
            ))?;
        
        // Add newline if not already present
        if !formatted.ends_with('\n') {
            stdout.write_all(b"\n")
                .map_err(|e| PluginError::ExecutionError(
                    format!("Failed to write newline to stdout: {}", e)
                ))?;
        }
        
        // Flush stdout
        stdout.flush()
            .map_err(|e| PluginError::ExecutionError(
                format!("Failed to flush stdout: {}", e)
            ))?;
        
        // Update statistics
        self.update_stats(1);
        
        Ok(())
    }
    
    /// Update statistics
    fn update_stats(&self, events_written: usize) {
        let mut stats = self.stats.lock().unwrap();
        let start_time = self.start_time.lock().unwrap();
        
        stats.events_written += events_written as u64;
        stats.write_time_ms = start_time.elapsed().as_millis() as u64;
    }
    
    /// Check if buffer needs flushing
    fn needs_flush(&self) -> bool {
        let buffer = self.write_buffer.lock().unwrap();
        let last_flush = self.last_flush.lock().unwrap();
        
        // Check buffer size
        if buffer.len() >= self.buffer_size {
            return true;
        }
        
        // Check time-based flushing
        if let Some(last_flush_time) = *last_flush {
            if last_flush_time.elapsed() >= Duration::from_millis(self.flush_interval) {
                return true;
            }
        }
        
        false
    }
    
    /// Flush buffer contents
    fn flush_buffer(&self) -> PluginResult<()> {
        let mut buffer = self.write_buffer.lock().unwrap();
        
        if buffer.is_empty() {
            return Ok(());
        }
        
        // Format and write all buffered events
        let formatted = self.formatter.format_batch(&buffer)
            .map_err(|e| PluginError::ExecutionError(
                format!("Failed to format batch: {}", e)
            ))?;
        
        self.write_formatted(&formatted)?;
        
        // Update statistics and state
        let events_written = buffer.len();
        buffer.clear();
        
        // Update last flush time
        let mut last_flush = self.last_flush.lock().unwrap();
        *last_flush = Some(Instant::now());
        
        self.update_stats(events_written);
        
        Ok(())
    }
}

impl Plugin for StdoutOutput {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn config(&self) -> &HashMap<String, Value> {
        &self.config
    }
    
    fn plugin_type(&self) -> PluginType {
        PluginType::Output
    }
    
    fn validate_config(&self) -> PluginResult<()> {
        // Validate buffer size
        if self.buffer_size == 0 {
            return Err(PluginError::ConfigurationError(
                "Buffer size must be greater than 0".to_string()
            ));
        }
        
        // Validate flush interval
        if self.flush_interval == 0 {
            return Err(PluginError::ConfigurationError(
                "Flush interval must be greater than 0".to_string()
            ));
        }
        
        // Validate buffer size limit
        if self.buffer_size > 1_000_000 {
            return Err(PluginError::ConfigurationError(
                "Buffer size cannot exceed 1,000,000".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn initialize(&mut self) -> PluginResult<()> {
        let mut start_time = self.start_time.lock().unwrap();
        *start_time = Instant::now();
        
        let mut last_write = self.last_write.lock().unwrap();
        *last_write = None;
        
        let mut last_flush = self.last_flush.lock().unwrap();
        *last_flush = None;
        
        // Clear buffer
        let mut buffer = self.write_buffer.lock().unwrap();
        buffer.clear();
        buffer.reserve(self.buffer_size);
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> PluginResult<()> {
        // Flush any remaining buffered data
        self.flush()?;
        
        // Update final statistics
        let start_time = self.start_time.lock().unwrap();
        let elapsed = start_time.elapsed();
        let mut stats = self.stats.lock().unwrap();
        stats.write_time_ms = elapsed.as_millis() as u64;
        
        Ok(())
    }
    
    fn status(&self) -> PluginStatus {
        PluginStatus::Ready
    }
}

impl Output for StdoutOutput {
    fn write(&self, event: Event) -> PluginResult<()> {
        // Format the event with ensured @timestamp
        let formatted = self.formatter.format_with_timestamp(&event)
            .map_err(|e| PluginError::ExecutionError(
                format!("Failed to format event: {}", e)
            ))?;
        
        // Write to stdout
        self.write_formatted(&formatted)?;
        
        // Update last write time
        let mut last_write = self.last_write.lock().unwrap();
        *last_write = Some(Instant::now());
        
        Ok(())
    }
    
    fn write_batch(&self, events: Vec<Event>) -> PluginResult<()> {
        if events.is_empty() {
            return Ok(());
        }
        
        // For small batches, write directly
        if events.len() <= 10 {
            for event in events {
                self.write(event)?;
            }
            return Ok(());
        }
        
        // For larger batches, use buffering
        let mut buffer = self.write_buffer.lock().unwrap();
        
        for event in events {
            buffer.push(event);
            
            // Check if buffer needs flushing
            if buffer.len() >= self.buffer_size {
                // Temporarily release lock to avoid deadlock
                let events_to_flush = buffer.split_off(0);
                drop(buffer); // Release lock
                
                // Format and write with ensured @timestamp
                let formatted = self.formatter.format_batch_with_timestamp(&events_to_flush)
                    .map_err(|e| PluginError::ExecutionError(
                        format!("Failed to format batch: {}", e)
                    ))?;
                
                self.write_formatted(&formatted)?;
                
                // Reacquire lock
                buffer = self.write_buffer.lock().unwrap();
            }
        }
        
        // Update last write time
        let mut last_write = self.last_write.lock().unwrap();
        *last_write = Some(Instant::now());
        
        Ok(())
    }
    
    fn flush(&self) -> PluginResult<()> {
        // This is a bit tricky because flush needs &mut self
        // We'll use interior mutability pattern
        let mut buffer = self.write_buffer.lock().unwrap();
        
        if !buffer.is_empty() {
            let events = std::mem::take(&mut *buffer);
            drop(buffer); // Release lock
            
            // Format and write with ensured @timestamp
            let formatted = self.formatter.format_batch_with_timestamp(&events)
                .map_err(|e| PluginError::ExecutionError(
                    format!("Failed to format batch during flush: {}", e)
                ))?;
            
            self.write_formatted(&formatted)?;
            
            // Update last flush time
            let mut last_flush = self.last_flush.lock().unwrap();
            *last_flush = Some(Instant::now());
        }
        
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        true // stdout is always ready
    }
    
    fn stats(&self) -> OutputStats {
        self.stats.lock().unwrap().clone()
    }
}

impl Default for StdoutOutput {
    fn default() -> Self {
        let config = PluginConfig {
            name: "stdout".to_string(),
            plugin_type: PluginType::Output,
            config: HashMap::new(),
        };
        
        Self::from_config(&config).unwrap()
    }
}