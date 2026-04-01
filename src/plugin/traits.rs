//! Plugin trait definitions

use crate::event::Event;
use crate::plugin::error::PluginResult;
use std::collections::HashMap;

/// Plugin type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginType {
    /// Input plugin (reads data from external sources)
    Input,
    
    /// Filter plugin (processes and transforms events)
    Filter,
    
    /// Output plugin (writes data to external destinations)
    Output,
}

impl PluginType {
    /// Get string representation of plugin type
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Filter => "filter",
            Self::Output => "output",
        }
    }
    
    /// Parse plugin type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "input" => Some(Self::Input),
            "filter" => Some(Self::Filter),
            "output" => Some(Self::Output),
            _ => None,
        }
    }
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Trait for plugins that can be created from configuration
pub trait FromConfig: Plugin {
    /// Create plugin from configuration
    fn from_config(config: &crate::plugin::PluginConfig) -> PluginResult<Self>
    where
        Self: Sized;
}

/// Base plugin trait
pub trait Plugin: Send + Sync {
    /// Get plugin name
    fn name(&self) -> &str;
    
    /// Get plugin configuration
    fn config(&self) -> &HashMap<String, serde_json::Value>;
    
    /// Get plugin type
    fn plugin_type(&self) -> PluginType;
    
    /// Validate plugin configuration
    fn validate_config(&self) -> PluginResult<()> {
        // Default implementation: always valid
        Ok(())
    }
    
    /// Initialize plugin (called after construction)
    fn initialize(&mut self) -> PluginResult<()> {
        // Default implementation: do nothing
        Ok(())
    }
    
    /// Shutdown plugin (cleanup resources)
    fn shutdown(&mut self) -> PluginResult<()> {
        // Default implementation: do nothing
        Ok(())
    }
    
    /// Get plugin status
    fn status(&self) -> PluginStatus {
        PluginStatus::Ready
    }
}

/// Input plugin trait
pub trait Input: Plugin {
    /// Read next event from input source
    /// Returns Ok(None) if no more events available (non-blocking)
    /// Returns Ok(Some(event)) if event was read
    /// Returns Err on error
    fn read(&mut self) -> PluginResult<Option<Event>>;
    
    /// Check if input is ready for reading
    fn is_ready(&self) -> bool {
        true
    }
    
    /// Get input statistics
    fn stats(&self) -> InputStats {
        InputStats::default()
    }
}

/// Filter plugin trait
pub trait Filter: Plugin {
    /// Process a single event
    fn process(&self, event: Event) -> PluginResult<Event>;
    
    /// Process multiple events (batch processing)
    fn process_batch(&self, events: Vec<Event>) -> PluginResult<Vec<Event>> {
        let mut results = Vec::with_capacity(events.len());
        
        for event in events {
            match self.process(event) {
                Ok(processed) => results.push(processed),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
    
    /// Check if filter can process events
    fn can_process(&self) -> bool {
        true
    }
    
    /// Get filter statistics
    fn stats(&self) -> FilterStats {
        FilterStats::default()
    }
}

/// Output plugin trait
pub trait Output: Plugin {
    /// Write a single event to output destination
    fn write(&self, event: Event) -> PluginResult<()>;
    
    /// Write multiple events (batch writing)
    fn write_batch(&self, events: Vec<Event>) -> PluginResult<()> {
        for event in events {
            self.write(event)?;
        }
        Ok(())
    }
    
    /// Flush any buffered data
    fn flush(&self) -> PluginResult<()>;
    
    /// Check if output is ready for writing
    fn is_ready(&self) -> bool {
        true
    }
    
    /// Get output statistics
    fn stats(&self) -> OutputStats {
        OutputStats::default()
    }
}

/// Plugin status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginStatus {
    /// Plugin is initializing
    Initializing,
    
    /// Plugin is ready for operation
    Ready,
    
    /// Plugin is running
    Running,
    
    /// Plugin is paused
    Paused,
    
    /// Plugin is stopping
    Stopping,
    
    /// Plugin has stopped
    Stopped,
    
    /// Plugin has encountered an error
    Error,
}

impl std::fmt::Display for PluginStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            Self::Initializing => "initializing",
            Self::Ready => "ready",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
            Self::Error => "error",
        };
        write!(f, "{}", status)
    }
}

/// Input plugin statistics
#[derive(Debug, Clone, Default)]
pub struct InputStats {
    /// Number of events read
    pub events_read: u64,
    
    /// Number of bytes read
    pub bytes_read: u64,
    
    /// Number of read errors
    pub errors: u64,
    
    /// Time spent reading (in milliseconds)
    pub read_time_ms: u64,
}

/// Filter plugin statistics
#[derive(Debug, Clone, Default)]
pub struct FilterStats {
    /// Number of events processed
    pub events_processed: u64,
    
    /// Number of events filtered out
    pub events_filtered: u64,
    
    /// Number of processing errors
    pub errors: u64,
    
    /// Time spent processing (in milliseconds)
    pub process_time_ms: u64,
}

/// Output plugin statistics
#[derive(Debug, Clone, Default)]
pub struct OutputStats {
    /// Number of events written
    pub events_written: u64,
    
    /// Number of bytes written
    pub bytes_written: u64,
    
    /// Number of write errors
    pub errors: u64,
    
    /// Time spent writing (in milliseconds)
    pub write_time_ms: u64,
}

/// Type alias for input plugin constructor
pub type InputConstructor = Box<dyn Fn() -> PluginResult<Box<dyn Input>> + Send + Sync>;

/// Type alias for filter plugin constructor
pub type FilterConstructor = Box<dyn Fn() -> PluginResult<Box<dyn Filter>> + Send + Sync>;

/// Type alias for output plugin constructor
pub type OutputConstructor = Box<dyn Fn() -> PluginResult<Box<dyn Output>> + Send + Sync>;