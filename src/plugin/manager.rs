//! Plugin manager for managing plugin instances

use crate::event::Event;
use crate::plugin::error::{PluginError, PluginResult};
use crate::plugin::traits::{Input, Filter, Output, PluginType};
use crate::plugin::{PluginConfig, PluginFactory};
use std::collections::HashMap;

/// Plugin manager for managing plugin instances
pub struct PluginManager {
    /// Input plugin instances
    inputs: HashMap<String, Box<dyn Input>>,
    
    /// Filter plugin instances
    filters: HashMap<String, Box<dyn Filter>>,
    
    /// Output plugin instances
    outputs: HashMap<String, Box<dyn Output>>,
    
    /// Plugin factory for creating instances
    factory: PluginFactory,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            filters: HashMap::new(),
            outputs: HashMap::new(),
            factory: PluginFactory::new(),
        }
    }
    
    /// Create a plugin manager with a custom factory
    pub fn with_factory(factory: PluginFactory) -> Self {
        Self {
            inputs: HashMap::new(),
            filters: HashMap::new(),
            outputs: HashMap::new(),
            factory,
        }
    }
    
    /// Add a plugin from configuration
    pub fn add_plugin(&mut self, config: PluginConfig) -> PluginResult<()> {
        // Validate configuration
        self.validate_config(&config)?;
        
        match config.plugin_type() {
            PluginType::Input => {
                let plugin = self.factory.create_input(&config)?;
                self.inputs.insert(config.name().to_string(), plugin);
            }
            PluginType::Filter => {
                let plugin = self.factory.create_filter(&config)?;
                self.filters.insert(config.name().to_string(), plugin);
            }
            PluginType::Output => {
                let plugin = self.factory.create_output(&config)?;
                self.outputs.insert(config.name().to_string(), plugin);
            }
        }
        
        Ok(())
    }
    
    /// Remove a plugin
    pub fn remove_plugin(&mut self, name: &str, plugin_type: PluginType) -> PluginResult<()> {
        match plugin_type {
            PluginType::Input => {
                if self.inputs.remove(name).is_none() {
                    return Err(PluginError::not_found(name));
                }
            }
            PluginType::Filter => {
                if self.filters.remove(name).is_none() {
                    return Err(PluginError::not_found(name));
                }
            }
            PluginType::Output => {
                if self.outputs.remove(name).is_none() {
                    return Err(PluginError::not_found(name));
                }
            }
        }
        
        Ok(())
    }
    
    /// Get an input plugin
    pub fn get_input(&self, name: &str) -> Option<&dyn Input> {
        self.inputs.get(name).map(|p| p.as_ref())
    }
    
    /// Get a mutable input plugin
    pub fn get_input_mut(&mut self, name: &str) -> Option<&mut dyn Input> {
        self.inputs.get_mut(name).map(|p| p.as_mut())
    }
    
    /// Get a filter plugin
    pub fn get_filter(&self, name: &str) -> Option<&dyn Filter> {
        self.filters.get(name).map(|p| p.as_ref())
    }
    
    /// Get an output plugin
    pub fn get_output(&self, name: &str) -> Option<&dyn Output> {
        self.outputs.get(name).map(|p| p.as_ref())
    }
    
    /// Get all input plugins
    pub fn inputs(&self) -> Vec<&dyn Input> {
        self.inputs.values().map(|p| p.as_ref()).collect()
    }
    
    /// Get all filter plugins
    pub fn filters(&self) -> Vec<&dyn Filter> {
        self.filters.values().map(|p| p.as_ref()).collect()
    }
    
    /// Get all output plugins
    pub fn outputs(&self) -> Vec<&dyn Output> {
        self.outputs.values().map(|p| p.as_ref()).collect()
    }
    
    /// Get plugin names by type
    pub fn plugin_names(&self, plugin_type: PluginType) -> Vec<String> {
        match plugin_type {
            PluginType::Input => self.inputs.keys().cloned().collect(),
            PluginType::Filter => self.filters.keys().cloned().collect(),
            PluginType::Output => self.outputs.keys().cloned().collect(),
        }
    }
    
    /// Check if a plugin exists
    pub fn has_plugin(&self, name: &str, plugin_type: PluginType) -> bool {
        match plugin_type {
            PluginType::Input => self.inputs.contains_key(name),
            PluginType::Filter => self.filters.contains_key(name),
            PluginType::Output => self.outputs.contains_key(name),
        }
    }
    
    /// Get number of input plugins
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }
    
    /// Get number of filter plugins
    pub fn filter_count(&self) -> usize {
        self.filters.len()
    }
    
    /// Get number of output plugins
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
    
    /// Get total number of plugins
    pub fn total_count(&self) -> usize {
        self.input_count() + self.filter_count() + self.output_count()
    }
    
    /// Initialize all plugins
    pub fn initialize_all(&mut self) -> PluginResult<()> {
        for plugin in self.inputs.values_mut() {
            plugin.initialize()?;
        }
        
        for plugin in self.filters.values_mut() {
            plugin.initialize()?;
        }
        
        for plugin in self.outputs.values_mut() {
            plugin.initialize()?;
        }
        
        Ok(())
    }
    
    /// Shutdown all plugins
    pub fn shutdown_all(&mut self) -> PluginResult<()> {
        let mut errors = Vec::new();
        
        for (name, plugin) in self.inputs.iter_mut() {
            if let Err(e) = plugin.shutdown() {
                errors.push(format!("Input plugin '{}': {}", name, e));
            }
        }
        
        for (name, plugin) in self.filters.iter_mut() {
            if let Err(e) = plugin.shutdown() {
                errors.push(format!("Filter plugin '{}': {}", name, e));
            }
        }
        
        for (name, plugin) in self.outputs.iter_mut() {
            if let Err(e) = plugin.shutdown() {
                errors.push(format!("Output plugin '{}': {}", name, e));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(PluginError::other(&errors.join("; ")))
        }
    }
    
    /// Process events through the pipeline
    pub fn process_pipeline(&self, events: Vec<Event>) -> PluginResult<Vec<Event>> {
        let mut processed_events = events;
        
        // Apply filters in order
        for filter in self.filters.values() {
            processed_events = filter.process_batch(processed_events)?;
        }
        
        // Write to outputs
        for output in self.outputs.values() {
            output.write_batch(processed_events.clone())?;
        }
        
        Ok(processed_events)
    }
    
    /// Read events from all inputs
    pub fn read_from_inputs(&mut self) -> PluginResult<Vec<Event>> {
        let mut events = Vec::new();
        
        for input in self.inputs.values_mut() {
            while let Some(event) = input.read()? {
                events.push(event);
            }
        }
        
        Ok(events)
    }
    
    /// Validate plugin configuration
    fn validate_config(&self, config: &PluginConfig) -> PluginResult<()> {
        // Check name is not empty
        if config.name().is_empty() {
            return Err(PluginError::invalid_name("Plugin name cannot be empty"));
        }
        
        // Check for duplicate names
        if self.has_plugin(config.name(), config.plugin_type()) {
            return Err(PluginError::registration_error(&format!(
                "Plugin '{}' of type '{}' already exists",
                config.name(),
                config.plugin_type()
            )));
        }
        
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginManager")
            .field("inputs", &self.plugin_names(PluginType::Input))
            .field("filters", &self.plugin_names(PluginType::Filter))
            .field("outputs", &self.plugin_names(PluginType::Output))
            .field("total", &self.total_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;
    use serde_json::json;
    
    struct TestInput {
        count: usize,
    }
    
    impl Input for TestInput {
        fn read(&mut self) -> PluginResult<Option<Event>> {
            if self.count < 3 {
                self.count += 1;
                Ok(Some(Event::new(json!({"id": self.count}))))
            } else {
                Ok(None)
            }
        }
        fn name(&self) -> &str { "test_input" }
        fn config(&self) -> &HashMap<String, serde_json::Value> { &HashMap::new() }
    }
    
    struct TestFilter;
    impl Filter for TestFilter {
        fn process(&self, event: Event) -> PluginResult<Event> {
            let mut event = event;
            event.set("filtered", json!(true));
            Ok(event)
        }
        fn name(&self) -> &str { "test_filter" }
        fn config(&self) -> &HashMap<String, serde_json::Value> { &HashMap::new() }
    }
    
    struct TestOutput {
        written: std::sync::Mutex<Vec<Event>>,
    }
    
    impl Output for TestOutput {
        fn write(&self, event: Event) -> PluginResult<()> {
            self.written.lock().unwrap().push(event);
            Ok(())
        }
        fn flush(&self) -> PluginResult<()> { Ok(()) }
        fn name(&self) -> &str { "test_output" }
        fn config(&self) -> &HashMap<String, serde_json::Value> { &HashMap::new() }
    }
    
    #[test]
    fn test_manager_basics() {
        let mut manager = PluginManager::new();
        
        // Add plugins
        let input_config = PluginConfig::new("input1", PluginType::Input);
        let filter_config = PluginConfig::new("filter1", PluginType::Filter);
        let output_config = PluginConfig::new("output1", PluginType::Output);
        
        manager.add_plugin(input_config).unwrap();
        manager.add_plugin(filter_config).unwrap();
        manager.add_plugin(output_config).unwrap();
        
        // Check counts
        assert_eq!(manager.input_count(), 1);
        assert_eq!(manager.filter_count(), 1);
        assert_eq!(manager.output_count(), 1);
        assert_eq!(manager.total_count(), 3);
        
        // Check existence
        assert!(manager.has_plugin("input1", PluginType::Input));
        assert!(manager.has_plugin("filter1", PluginType::Filter));
        assert!(manager.has_plugin("output1", PluginType::Output));
        
        // Get plugins
        assert!(manager.get_input("input1").is_some());
        assert!(manager.get_filter("filter1").is_some());
        assert!(manager.get_output("output1").is_some());
        
        // Get all plugins
        assert_eq!(manager.inputs().len(), 1);
        assert_eq!(manager.filters().len(), 1);
        assert_eq!(manager.outputs().len(), 1);
        
        // Get names
        assert_eq!(manager.plugin_names(PluginType::Input), vec!["input1".to_string()]);
    }
    
    #[test]
    fn test_manager_remove() {
        let mut manager = PluginManager::new();
        
        let config = PluginConfig::new("test", PluginType::Input);
        manager.add_plugin(config).unwrap();
        
        assert!(manager.has_plugin("test", PluginType::Input));
        
        manager.remove_plugin("test", PluginType::Input).unwrap();
        assert!(!manager.has_plugin("test", PluginType::Input));
        
        // Remove non-existent should fail
        let result = manager.remove_plugin("nonexistent", PluginType::Input);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_manager_duplicate() {
        let mut manager = PluginManager::new();
        
        let config1 = PluginConfig::new("test", PluginType::Input);
        manager.add_plugin(config1).unwrap();
        
        let config2 = PluginConfig::new("test", PluginType::Input);
        let result = manager.add_plugin(config2);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::RegistrationError(_)));
    }
    
    #[test]
    fn test_manager_validation() {
        let mut manager = PluginManager::new();
        
        // Empty name should fail
        let config = PluginConfig::new("", PluginType::Input);
        let result = manager.add_plugin(config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::InvalidName(_)));
    }
}