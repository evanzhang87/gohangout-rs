//! Plugin registry for registering and discovering plugins

use crate::plugin::error::{PluginError, PluginResult};
use crate::plugin::traits::{
    Input, Filter, Output, InputConstructor, FilterConstructor, OutputConstructor, PluginType,
};
use crate::plugin::PluginConfig;
use std::collections::HashMap;

/// Plugin registry for managing plugin registrations
#[derive(Default)]
pub struct PluginRegistry {
    /// Registered input plugins
    inputs: HashMap<String, InputConstructor>,
    
    /// Registered filter plugins
    filters: HashMap<String, FilterConstructor>,
    
    /// Registered output plugins
    outputs: HashMap<String, OutputConstructor>,
}

impl PluginRegistry {
    /// Create a new empty plugin registry
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            filters: HashMap::new(),
            outputs: HashMap::new(),
        }
    }
    
    /// Register an input plugin
    pub fn register_input<F>(&mut self, name: &str, constructor: F)
    where
        F: Fn() -> PluginResult<Box<dyn Input>> + Send + Sync + 'static,
    {
        self.inputs.insert(name.to_string(), Box::new(constructor));
    }
    
    /// Register a filter plugin
    pub fn register_filter<F>(&mut self, name: &str, constructor: F)
    where
        F: Fn() -> PluginResult<Box<dyn Filter>> + Send + Sync + 'static,
    {
        self.filters.insert(name.to_string(), Box::new(constructor));
    }
    
    /// Register an output plugin
    pub fn register_output<F>(&mut self, name: &str, constructor: F)
    where
        F: Fn() -> PluginResult<Box<dyn Output>> + Send + Sync + 'static,
    {
        self.outputs.insert(name.to_string(), Box::new(constructor));
    }
    
    /// Create an input plugin instance
    pub fn create_input(&self, name: &str) -> PluginResult<Box<dyn Input>> {
        self.inputs
            .get(name)
            .ok_or_else(|| PluginError::not_found(name))?
            ()
    }
    
    /// Create a filter plugin instance
    pub fn create_filter(&self, name: &str) -> PluginResult<Box<dyn Filter>> {
        self.filters
            .get(name)
            .ok_or_else(|| PluginError::not_found(name))?
            ()
    }
    
    /// Create an output plugin instance
    pub fn create_output(&self, name: &str) -> PluginResult<Box<dyn Output>> {
        self.outputs
            .get(name)
            .ok_or_else(|| PluginError::not_found(name))?
            ()
    }
    
    /// Check if an input plugin is registered
    pub fn has_input(&self, name: &str) -> bool {
        self.inputs.contains_key(name)
    }
    
    /// Check if a filter plugin is registered
    pub fn has_filter(&self, name: &str) -> bool {
        self.filters.contains_key(name)
    }
    
    /// Check if an output plugin is registered
    pub fn has_output(&self, name: &str) -> bool {
        self.outputs.contains_key(name)
    }
    
    /// Check if any plugin of any type is registered with the given name
    pub fn has_plugin(&self, name: &str) -> bool {
        self.has_input(name) || self.has_filter(name) || self.has_output(name)
    }
    
    /// Get registered input plugin names
    pub fn input_names(&self) -> Vec<String> {
        self.inputs.keys().cloned().collect()
    }
    
    /// Get registered filter plugin names
    pub fn filter_names(&self) -> Vec<String> {
        self.filters.keys().cloned().collect()
    }
    
    /// Get registered output plugin names
    pub fn output_names(&self) -> Vec<String> {
        self.outputs.keys().cloned().collect()
    }
    
    /// Get all registered plugin names
    pub fn all_plugin_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        names.extend(self.input_names());
        names.extend(self.filter_names());
        names.extend(self.output_names());
        names
    }
    
    /// Get plugin type by name
    pub fn plugin_type(&self, name: &str) -> Option<PluginType> {
        if self.has_input(name) {
            Some(PluginType::Input)
        } else if self.has_filter(name) {
            Some(PluginType::Filter)
        } else if self.has_output(name) {
            Some(PluginType::Output)
        } else {
            None
        }
    }
    
    /// Unregister an input plugin
    pub fn unregister_input(&mut self, name: &str) -> bool {
        self.inputs.remove(name).is_some()
    }
    
    /// Unregister a filter plugin
    pub fn unregister_filter(&mut self, name: &str) -> bool {
        self.filters.remove(name).is_some()
    }
    
    /// Unregister an output plugin
    pub fn unregister_output(&mut self, name: &str) -> bool {
        self.outputs.remove(name).is_some()
    }
    
    /// Unregister a plugin of any type
    pub fn unregister_plugin(&mut self, name: &str) -> bool {
        let mut removed = false;
        removed |= self.unregister_input(name);
        removed |= self.unregister_filter(name);
        removed |= self.unregister_output(name);
        removed
    }
    
    /// Clear all registered plugins
    pub fn clear(&mut self) {
        self.inputs.clear();
        self.filters.clear();
        self.outputs.clear();
    }
    
    /// Get number of registered input plugins
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }
    
    /// Get number of registered filter plugins
    pub fn filter_count(&self) -> usize {
        self.filters.len()
    }
    
    /// Get number of registered output plugins
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
    
    /// Get total number of registered plugins
    pub fn total_count(&self) -> usize {
        self.input_count() + self.filter_count() + self.output_count()
    }
    
    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.total_count() == 0
    }
}

impl std::fmt::Debug for PluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginRegistry")
            .field("inputs", &self.input_names())
            .field("filters", &self.filter_names())
            .field("outputs", &self.output_names())
            .field("total", &self.total_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;
    use crate::plugin::traits::Plugin;
    use serde_json::json;
    
    struct TestInput;
    impl Plugin for TestInput {
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
    impl Input for TestInput {
        fn read(&mut self) -> PluginResult<Option<Event>> {
            Ok(Some(Event::new(json!({"test": "input"}))))
        }
    }
    
    struct TestFilter;
    impl Plugin for TestFilter {
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
    impl Filter for TestFilter {
        fn process(&self, event: Event) -> PluginResult<Event> {
            let mut event = event;
            event.set("filtered", json!(true));
            Ok(event)
        }
    }
    
    struct TestOutput;
    impl Plugin for TestOutput {
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
    impl Output for TestOutput {
        fn write(&self, _event: Event) -> PluginResult<()> { Ok(()) }
        fn flush(&self) -> PluginResult<()> { Ok(()) }
    }
    
    #[test]
    fn test_registry_basics() {
        let mut registry = PluginRegistry::new();
        
        // Register plugins
        registry.register_input("stdin", || Ok(Box::new(TestInput) as Box<dyn Input>));
        registry.register_filter("add", || Ok(Box::new(TestFilter) as Box<dyn Filter>));
        registry.register_output("stdout", || Ok(Box::new(TestOutput) as Box<dyn Output>));
        
        // Check registrations
        assert!(registry.has_input("stdin"));
        assert!(registry.has_filter("add"));
        assert!(registry.has_output("stdout"));
        assert!(registry.has_plugin("stdin"));
        assert!(registry.has_plugin("add"));
        assert!(registry.has_plugin("stdout"));
        
        // Create instances
        let input = registry.create_input("stdin").unwrap();
        let filter = registry.create_filter("add").unwrap();
        let output = registry.create_output("stdout").unwrap();
        
        assert_eq!(input.name(), "test_input");
        assert_eq!(filter.name(), "test_filter");
        assert_eq!(output.name(), "test_output");
        
        // Check counts
        assert_eq!(registry.input_count(), 1);
        assert_eq!(registry.filter_count(), 1);
        assert_eq!(registry.output_count(), 1);
        assert_eq!(registry.total_count(), 3);
        assert!(!registry.is_empty());
        
        // Get names
        assert_eq!(registry.input_names(), vec!["stdin".to_string()]);
        assert_eq!(registry.filter_names(), vec!["add".to_string()]);
        assert_eq!(registry.output_names(), vec!["stdout".to_string()]);
        
        // Get plugin type
        assert_eq!(registry.plugin_type("stdin"), Some(PluginType::Input));
        assert_eq!(registry.plugin_type("add"), Some(PluginType::Filter));
        assert_eq!(registry.plugin_type("stdout"), Some(PluginType::Output));
        assert_eq!(registry.plugin_type("unknown"), None);
    }
    
    #[test]
    fn test_registry_unregister() {
        let mut registry = PluginRegistry::new();
        
        registry.register_input("test", || Ok(Box::new(TestInput) as Box<dyn Input>));
        assert!(registry.has_input("test"));
        
        assert!(registry.unregister_input("test"));
        assert!(!registry.has_input("test"));
        
        assert!(!registry.unregister_input("nonexistent"));
    }
    
    #[test]
    fn test_registry_clear() {
        let mut registry = PluginRegistry::new();
        
        registry.register_input("in", || Ok(Box::new(TestInput) as Box<dyn Input>));
        registry.register_filter("filter", || Ok(Box::new(TestFilter) as Box<dyn Filter>));
        
        assert_eq!(registry.total_count(), 2);
        
        registry.clear();
        assert!(registry.is_empty());
        assert_eq!(registry.total_count(), 0);
    }
    
    #[test]
    fn test_registry_error_handling() {
        let registry = PluginRegistry::new();
        
        // Try to create non-existent plugin
        let result = registry.create_input("nonexistent");
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, PluginError::NotFound(_)));
        }
    }
}