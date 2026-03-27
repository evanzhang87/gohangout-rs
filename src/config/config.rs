//! Configuration structures for Gohangout-rs

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Main application configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AppConfig {
    /// List of input configurations
    pub inputs: Vec<InputConfig>,
    
    /// List of filter configurations
    pub filters: Vec<FilterConfig>,
    
    /// List of output configurations
    pub outputs: Vec<OutputConfig>,
    
    /// Number of worker threads for processing
    #[serde(default = "default_workers")]
    pub workers: usize,
    
    /// Batch size for processing events
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    
    /// Buffer size for internal channels
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            inputs: Vec::new(),
            filters: Vec::new(),
            outputs: Vec::new(),
            workers: default_workers(),
            batch_size: default_batch_size(),
            buffer_size: default_buffer_size(),
        }
    }
}

fn default_workers() -> usize {
    crate::config::defaults::DEFAULT_WORKERS
}

fn default_batch_size() -> usize {
    crate::config::defaults::DEFAULT_BATCH_SIZE
}

fn default_buffer_size() -> usize {
    crate::config::defaults::DEFAULT_BUFFER_SIZE
}

/// Input plugin configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputConfig {
    /// Type of input plugin (e.g., "kafka", "stdin", "tcp", "udp")
    pub r#type: String,
    
    /// Plugin-specific configuration
    pub config: HashMap<String, Value>,
}

/// Filter plugin configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilterConfig {
    /// Type of filter plugin (e.g., "add", "drop", "grok", "date")
    pub r#type: String,
    
    /// Plugin-specific configuration
    pub config: HashMap<String, Value>,
    
    /// Optional condition for applying this filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

/// Output plugin configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputConfig {
    /// Type of output plugin (e.g., "elasticsearch", "clickhouse", "kafka", "stdout")
    pub r#type: String,
    
    /// Plugin-specific configuration
    pub config: HashMap<String, Value>,
}

impl AppConfig {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.workers == 0 {
            return Err("workers must be greater than 0".to_string());
        }
        
        if self.batch_size == 0 {
            return Err("batch_size must be greater than 0".to_string());
        }
        
        if self.buffer_size == 0 {
            return Err("buffer_size must be greater than 0".to_string());
        }
        
        // Validate that at least one input and output is configured
        if self.inputs.is_empty() {
            return Err("at least one input must be configured".to_string());
        }
        
        if self.outputs.is_empty() {
            return Err("at least one output must be configured".to_string());
        }
        
        Ok(())
    }
    
    /// Check if configuration is empty (no inputs/outputs)
    pub fn is_empty(&self) -> bool {
        self.inputs.is_empty() && self.outputs.is_empty()
    }
    
    /// Get the total number of plugins configured
    pub fn total_plugins(&self) -> usize {
        self.inputs.len() + self.filters.len() + self.outputs.len()
    }
}