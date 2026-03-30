//! RandomInput plugin implementation (simplified to match GoHangout)
//!
//! This implementation follows the exact logic and configuration format
//! from the original GoHangout project (github.com/childe/gohangout).

use crate::event::Event;
use crate::prelude::{PluginError, PluginResult, Input, InputStats, Plugin, PluginType};
use crate::plugin::PluginConfig;
use crate::plugin::traits::PluginStatus;
use rand::Rng;
use serde_json::Value;
use std::collections::HashMap;

/// RandomInput plugin that generates random numbers
///
/// Configuration (matches GoHangout exactly):
/// ```yaml
/// type: Random
/// config:
///   from: 1          # required: lower bound of random range
///   to: 100          # required: upper bound of random range
///   max_messages: -1 # optional: max messages to generate (-1 = unlimited)
/// ```
#[derive(Debug)]
pub struct RandomInput {
    /// Plugin name
    name: String,
    
    /// Plugin configuration
    config: HashMap<String, Value>,
    
    /// Lower bound of random range
    from: i64,
    
    /// Upper bound of random range
    to: i64,
    
    /// Maximum messages to generate (-1 = unlimited)
    max_messages: i64,
    
    /// Number of messages generated so far
    count: i64,
    
    /// Statistics
    stats: InputStats,
}

impl RandomInput {
    /// Create a new RandomInput from configuration
    pub fn from_config(config: &PluginConfig) -> PluginResult<Self> {
        let name = config.name.clone();
        let config_map = config.config.clone();
        
        // Get 'from' parameter (required)
        let from = config_map.get("from")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| PluginError::ConfigurationError(
                "from must be configured in Random Input".to_string()
            ))?;
        
        // Get 'to' parameter (required)
        let to = config_map.get("to")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| PluginError::ConfigurationError(
                "to must be configured in Random Input".to_string()
            ))?;
        
        // Validate range
        if from > to {
            return Err(PluginError::ConfigurationError(
                format!("from ({}) must be less than or equal to to ({})", from, to)
            ));
        }
        
        // Get 'max_messages' parameter (optional, default -1)
        let max_messages = config_map.get("max_messages")
            .and_then(|v| v.as_i64())
            .unwrap_or(-1);
        
        Ok(Self {
            name,
            config: config_map,
            from,
            to,
            max_messages,
            count: 0,
            stats: InputStats::default(),
        })
    }
    
    /// Generate a random number in the range [from, to]
    fn generate_random_number(&self) -> i64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(self.from..=self.to)
    }
}

impl Plugin for RandomInput {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn config(&self) -> &HashMap<String, Value> {
        &self.config
    }
    
    fn plugin_type(&self) -> PluginType {
        PluginType::Input
    }
    
    fn validate_config(&self) -> PluginResult<()> {
        // Already validated in from_config
        Ok(())
    }
    
    fn initialize(&mut self) -> PluginResult<()> {
        self.count = 0;
        self.stats = InputStats::default();
        Ok(())
    }
    
    fn shutdown(&mut self) -> PluginResult<()> {
        Ok(())
    }
    
    fn status(&self) -> PluginStatus {
        if self.max_messages >= 0 && self.count >= self.max_messages {
            PluginStatus::Stopped
        } else {
            PluginStatus::Ready
        }
    }
}

impl Input for RandomInput {
    fn read(&mut self) -> PluginResult<Option<Event>> {
        // Check if we've reached max messages
        if self.max_messages >= 0 && self.count >= self.max_messages {
            return Ok(None);
        }
        
        // Generate random number
        let random_number = self.generate_random_number();
        
        // Create event with the number as a string (matching GoHangout's plain decoder)
        let mut data = HashMap::new();
        data.insert("message".to_string(), Value::String(random_number.to_string()));
        
        // Update statistics
        self.count += 1;
        self.stats.events_read += 1;
        // Note: bytes_read is not tracked in this simple implementation
        // Note: read_time_ms is not tracked in this simple implementation
        
        Ok(Some(Event::new(Value::Object(data.into_iter().collect()))))
    }
    
    fn stats(&self) -> InputStats {
        self.stats.clone()
    }
}

impl Default for RandomInput {
    fn default() -> Self {
        let mut config = PluginConfig::new("random", PluginType::Input);
        config.config = {
            let mut map = HashMap::new();
            map.insert("from".to_string(), Value::from(1));
            map.insert("to".to_string(), Value::from(100));
            map
        };
        
        Self::from_config(&config).unwrap()
    }
}