//! Plugin factory for creating plugin instances from configuration
#![allow(dead_code)]

use crate::event::Event;
use crate::plugin::error::{PluginError, PluginResult};
use crate::plugin::traits::{Input, Filter, Output, PluginType};
use crate::plugin::{PluginConfig, PluginRegistry};
use std::collections::HashMap;

/// Plugin factory for creating plugin instances
pub struct PluginFactory {
    /// Plugin registry for looking up plugin constructors
    registry: PluginRegistry,
}

impl PluginFactory {
    /// Create a new plugin factory
    pub fn new() -> Self {
        Self {
            registry: PluginRegistry::new(),
        }
    }
    
    /// Create a new plugin factory with a custom registry
    pub fn with_registry(registry: PluginRegistry) -> Self {
        Self { registry }
    }
    
    /// Get the plugin registry
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }
    
    /// Get mutable access to the plugin registry
    pub fn registry_mut(&mut self) -> &mut PluginRegistry {
        &mut self.registry
    }
    
    /// Create an input plugin from configuration
    pub fn create_input(&self, config: &PluginConfig) -> PluginResult<Box<dyn Input>> {
        // Validate configuration
        self.validate_config(config)?;
        
        // Create plugin instance
        let mut plugin = self.registry.create_input(config.name())?;
        
        // Initialize plugin
        plugin.initialize()?;
        
        Ok(plugin)
    }
    
    /// Create a filter plugin from configuration
    pub fn create_filter(&self, config: &PluginConfig) -> PluginResult<Box<dyn Filter>> {
        // Validate configuration
        self.validate_config(config)?;
        
        // Create plugin instance
        let mut plugin = self.registry.create_filter(config.name())?;
        
        // Initialize plugin
        plugin.initialize()?;
        
        Ok(plugin)
    }
    
    /// Create an output plugin from configuration
    pub fn create_output(&self, config: &PluginConfig) -> PluginResult<Box<dyn Output>> {
        // Validate configuration
        self.validate_config(config)?;
        
        // Create plugin instance
        let mut plugin = self.registry.create_output(config.name())?;
        
        // Initialize plugin
        plugin.initialize()?;
        
        Ok(plugin)
    }
    
    /// Create a plugin from configuration (type determined by config)
    pub fn create_plugin(&self, config: &PluginConfig) -> PluginResult<Box<dyn std::any::Any>> {
        match config.plugin_type() {
            PluginType::Input => {
                let plugin = self.create_input(config)?;
                Ok(Box::new(plugin) as Box<dyn std::any::Any>)
            }
            PluginType::Filter => {
                let plugin = self.create_filter(config)?;
                Ok(Box::new(plugin) as Box<dyn std::any::Any>)
            }
            PluginType::Output => {
                let plugin = self.create_output(config)?;
                Ok(Box::new(plugin) as Box<dyn std::any::Any>)
            }
        }
    }
    
    /// Register a built-in input plugin
    pub fn register_input<F>(&mut self, name: &str, constructor: F)
    where
        F: Fn() -> PluginResult<Box<dyn Input>> + Send + Sync + 'static,
    {
        self.registry.register_input(name, constructor);
    }
    
    /// Register a built-in filter plugin
    pub fn register_filter<F>(&mut self, name: &str, constructor: F)
    where
        F: Fn() -> PluginResult<Box<dyn Filter>> + Send + Sync + 'static,
    {
        self.registry.register_filter(name, constructor);
    }
    
    /// Register a built-in output plugin
    pub fn register_output<F>(&mut self, name: &str, constructor: F)
    where
        F: Fn() -> PluginResult<Box<dyn Output>> + Send + Sync + 'static,
    {
        self.registry.register_output(name, constructor);
    }
    
    /// Check if a plugin type is supported
    pub fn supports_plugin(&self, name: &str, plugin_type: PluginType) -> bool {
        match plugin_type {
            PluginType::Input => self.registry.has_input(name),
            PluginType::Filter => self.registry.has_filter(name),
            PluginType::Output => self.registry.has_output(name),
        }
    }
    
    /// Get supported plugin names by type
    pub fn supported_plugins(&self, plugin_type: PluginType) -> Vec<String> {
        match plugin_type {
            PluginType::Input => self.registry.input_names(),
            PluginType::Filter => self.registry.filter_names(),
            PluginType::Output => self.registry.output_names(),
        }
    }
    
    /// Validate plugin configuration
    fn validate_config(&self, config: &PluginConfig) -> PluginResult<()> {
        // Check name is not empty
        if config.name().is_empty() {
            return Err(PluginError::invalid_name("Plugin name cannot be empty"));
        }
        
        // Check plugin is supported
        if !self.supports_plugin(config.name(), config.plugin_type()) {
            return Err(PluginError::not_found(&format!(
                "Plugin '{}' of type '{}' is not supported",
                config.name(),
                config.plugin_type()
            )));
        }
        
        Ok(())
    }
    

}

impl Default for PluginFactory {
    fn default() -> Self {
        Self::new()
    }
}

// Built-in plugin implementations

// Note: StdinInput is now defined in src/input/stdin.rs
// We'll use the one from the input module

/// Standard output plugin (writes to stdout)
pub struct StdoutOutput {
    name: String,
    config: HashMap<String, serde_json::Value>,
}

impl StdoutOutput {
    /// Create a new stdout output plugin
    pub fn new() -> Self {
        Self {
            name: "stdout".to_string(),
            config: HashMap::new(),
        }
    }
}

impl crate::plugin::Plugin for StdoutOutput {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn config(&self) -> &HashMap<String, serde_json::Value> {
        &self.config
    }
    
    fn plugin_type(&self) -> crate::plugin::PluginType {
        crate::plugin::PluginType::Output
    }
    
    fn initialize(&mut self) -> PluginResult<()> {
        Ok(())
    }
    
    fn shutdown(&mut self) -> PluginResult<()> {
        Ok(())
    }
    
    fn validate_config(&self) -> PluginResult<()> {
        Ok(())
    }
}

impl Output for StdoutOutput {
    fn write(&self, event: Event) -> PluginResult<()> {
        // Simplified implementation
        // In a real implementation, this would write to stdout
        println!("{}", event.to_json()?);
        Ok(())
    }
    
    fn flush(&self) -> PluginResult<()> {
        Ok(())
    }
}

/// Add field filter plugin
pub struct AddFieldFilter {
    name: String,
    config: HashMap<String, serde_json::Value>,
    field: String,
    value: serde_json::Value,
}

impl AddFieldFilter {
    /// Create a new add field filter
    pub fn new(field: &str, value: serde_json::Value) -> Self {
        let mut config = HashMap::new();
        config.insert("field".to_string(), serde_json::Value::String(field.to_string()));
        config.insert("value".to_string(), value.clone());
        
        Self {
            name: format!("add_field_{}", field),
            config,
            field: field.to_string(),
            value,
        }
    }
}

impl crate::plugin::Plugin for AddFieldFilter {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn config(&self) -> &HashMap<String, serde_json::Value> {
        &self.config
    }
    
    fn plugin_type(&self) -> crate::plugin::PluginType {
        crate::plugin::PluginType::Filter
    }
    
    fn initialize(&mut self) -> PluginResult<()> {
        Ok(())
    }
    
    fn shutdown(&mut self) -> PluginResult<()> {
        Ok(())
    }
    
    fn validate_config(&self) -> PluginResult<()> {
        Ok(())
    }
}

impl Filter for AddFieldFilter {
    fn process(&self, mut event: Event) -> PluginResult<Event> {
        event.set(&self.field, self.value.clone());
        Ok(event)
    }
}

/// Default plugin factory with built-in plugins
pub fn default_factory() -> PluginFactory {
    let mut factory = PluginFactory::new();
    
    // Register built-in input plugins
    factory.register_input("stdin", || {
        Ok(Box::new(crate::input::StdinInput::default()) as Box<dyn Input>)
    });
    
    // Register built-in filter plugins
    factory.register_filter("add_field", || {
        Ok(Box::new(AddFieldFilter::new(
            "default_field",
            serde_json::Value::String("default_value".to_string()),
        )) as Box<dyn Filter>)
    });
    
    // Register built-in output plugins
    factory.register_output("stdout", || {
        Ok(Box::new(StdoutOutput::new()) as Box<dyn Output>)
    });
    
    factory
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    struct TestInputImpl;
    impl Plugin for TestInputImpl {
        fn name(&self) -> &str { "test_input" }
        fn config(&self) -> &HashMap<String, serde_json::Value> { 
            static CONFIG: std::sync::OnceLock<HashMap<String, serde_json::Value>> = std::sync::OnceLock::new();
            CONFIG.get_or_init(|| HashMap::new())
        }
        fn plugin_type(&self) -> PluginType { PluginType::Input }
        fn initialize(&mut self) -> PluginResult<()> { Ok(()) }
        fn shutdown(&mut self) -> PluginResult<()> { Ok(()) }
        fn validate_config(&self) -> PluginResult<()> { Ok(()) }
    }
    impl Input for TestInputImpl {
        fn read(&mut self) -> PluginResult<Option<Event>> {
            Ok(Some(Event::new(json!({"test": "input"}))))
        }
    }
    
    struct TestFilterImpl;
    impl Plugin for TestFilterImpl {
        fn name(&self) -> &str { "test_filter" }
        fn config(&self) -> &HashMap<String, serde_json::Value> { 
            static CONFIG: std::sync::OnceLock<HashMap<String, serde_json::Value>> = std::sync::OnceLock::new();
            CONFIG.get_or_init(|| HashMap::new())
        }
        fn plugin_type(&self) -> PluginType { PluginType::Filter }
        fn initialize(&mut self) -> PluginResult<()> { Ok(()) }
        fn shutdown(&mut self) -> PluginResult<()> { Ok(()) }
        fn validate_config(&self) -> PluginResult<()> { Ok(()) }
    }
    impl Filter for TestFilterImpl {
        fn process(&self, event: Event) -> PluginResult<Event> {
            let mut event = event;
            event.set("processed", json!(true));
            Ok(event)
        }
    }
    
    struct TestOutputImpl;
    impl Plugin for TestOutputImpl {
        fn name(&self) -> &str { "test_output" }
        fn config(&self) -> &HashMap<String, serde_json::Value> { 
            static CONFIG: std::sync::OnceLock<HashMap<String, serde_json::Value>> = std::sync::OnceLock::new();
            CONFIG.get_or_init(|| HashMap::new())
        }
        fn plugin_type(&self) -> PluginType { PluginType::Output }
        fn initialize(&mut self) -> PluginResult<()> { Ok(()) }
        fn shutdown(&mut self) -> PluginResult<()> { Ok(()) }
        fn validate_config(&self) -> PluginResult<()> { Ok(()) }
    }
    impl Output for TestOutputImpl {
        fn write(&self, _event: Event) -> PluginResult<()> { Ok(()) }
        fn flush(&self) -> PluginResult<()> { Ok(()) }
    }
    
    #[test]
    fn test_factory_basics() {
        let mut factory = PluginFactory::new();
        
        // Register test plugins
        factory.register_input("test_input", || Ok(Box::new(TestInputImpl) as Box<dyn Input>));
        factory.register_filter("test_filter", || Ok(Box::new(TestFilterImpl) as Box<dyn Filter>));
        factory.register_output("test_output", || Ok(Box::new(TestOutputImpl) as Box<dyn Output>));
        
        // Create plugins from config
        let input_config = PluginConfig::new("test_input", PluginType::Input);
        let filter_config = PluginConfig::new("test_filter", PluginType::Filter);
        let output_config = PluginConfig::new("test_output", PluginType::Output);
        
        let input = factory.create_input(&input_config).unwrap();
        let filter = factory.create_filter(&filter_config).unwrap();
        let output = factory.create_output(&output_config).unwrap();
        
        assert_eq!(input.name(), "test_input");
        assert_eq!(filter.name(), "test_filter");
        assert_eq!(output.name(), "test_output");
        
        // Check support
        assert!(factory.supports_plugin("test_input", PluginType::Input));
        assert!(factory.supports_plugin("test_filter", PluginType::Filter));
        assert!(factory.supports_plugin("test_output", PluginType::Output));
        
        // Get supported plugins
        assert!(factory.supported_plugins(PluginType::Input).contains(&"test_input".to_string()));
        assert!(factory.supported_plugins(PluginType::Filter).contains(&"test_filter".to_string()));
        assert!(factory.supported_plugins(PluginType::Output).contains(&"test_output".to_string()));
    }
    
    #[test]
    fn test_factory_validation() {
        let factory = PluginFactory::new();
        
        // Unknown plugin should fail
        let config = PluginConfig::new("unknown", PluginType::Input);
        let result = factory.create_input(&config);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, PluginError::NotFound(_)));
        }
        
        // Empty name should fail
        let config = PluginConfig::new("", PluginType::Input);
        let result = factory.create_input(&config);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, PluginError::InvalidName(_)));
        }
    }
    
    #[test]
    fn test_default_factory() {
        let factory = default_factory();
        
        // Check built-in plugins are registered
        assert!(factory.supports_plugin("stdin", PluginType::Input));
        assert!(factory.supports_plugin("add_field", PluginType::Filter));
        assert!(factory.supports_plugin("stdout", PluginType::Output));
        
        // Create built-in plugins
        let input_config = PluginConfig::new("stdin", PluginType::Input);
        let filter_config = PluginConfig::new("add_field", PluginType::Filter);
        let output_config = PluginConfig::new("stdout", PluginType::Output);
        
        let input = factory.create_input(&input_config).unwrap();
        let filter = factory.create_filter(&filter_config).unwrap();
        let output = factory.create_output(&output_config).unwrap();
        
        assert_eq!(input.name(), "stdin");
        assert_eq!(filter.name(), "add_field_default_field");
        assert_eq!(output.name(), "stdout");
    }
}