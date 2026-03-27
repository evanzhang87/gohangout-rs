//! Plugin system for GoHangout-rs
//!
//! This module defines the plugin architecture for input, filter, and output
//! plugins, along with registration and management utilities.

pub mod traits;
pub mod error;
mod registry;
mod manager;
mod factory;

pub use traits::{Input, Filter, Output, Plugin, PluginType};
pub use error::{PluginError, PluginResult};
pub use registry::PluginRegistry;
pub use manager::PluginManager;
pub use factory::PluginFactory;

/// Re-exports for convenience
pub mod prelude {
    pub use super::{Input, Filter, Output, Plugin, PluginType};
    pub use super::{PluginError, PluginResult, PluginRegistry, PluginManager, PluginFactory};
}

/// Plugin configuration structure
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// Plugin name (must be unique within its type)
    pub name: String,
    
    /// Plugin type (input, filter, or output)
    pub plugin_type: PluginType,
    
    /// Plugin-specific configuration
    pub config: std::collections::HashMap<String, serde_json::Value>,
}

impl PluginConfig {
    /// Create a new plugin configuration
    pub fn new(name: &str, plugin_type: PluginType) -> Self {
        Self {
            name: name.to_string(),
            plugin_type,
            config: std::collections::HashMap::new(),
        }
    }
    
    /// Create a plugin configuration with specific config
    pub fn with_config(
        name: &str,
        plugin_type: PluginType,
        config: std::collections::HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            name: name.to_string(),
            plugin_type,
            config,
        }
    }
    
    /// Get plugin name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get plugin type
    pub fn plugin_type(&self) -> PluginType {
        self.plugin_type
    }
    
    /// Get plugin configuration
    pub fn config(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.config
    }
    
    /// Get mutable access to plugin configuration
    pub fn config_mut(&mut self) -> &mut std::collections::HashMap<String, serde_json::Value> {
        &mut self.config
    }
    
    /// Set a configuration value
    pub fn set_config(&mut self, key: &str, value: serde_json::Value) {
        self.config.insert(key.to_string(), value);
    }
    
    /// Get a configuration value
    pub fn get_config(&self, key: &str) -> Option<&serde_json::Value> {
        self.config.get(key)
    }
}