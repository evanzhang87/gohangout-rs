//! Plugin system tests

use gohangout_rs::plugin::*;
use gohangout_rs::event::Event;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_input_trait_basic() {
    struct TestInput;
    
    impl Input for TestInput {
        fn read(&mut self) -> Result<Option<Event>, PluginError> {
            Ok(Some(Event::new(json!({"test": "data"}))))
        }
        
        fn name(&self) -> &str {
            "test_input"
        }
        
        fn config(&self) -> &HashMap<String, serde_json::Value> {
            &HashMap::new()
        }
    }
    
    let mut input = TestInput;
    let event = input.read().unwrap().unwrap();
    
    assert_eq!(event.get("test").unwrap(), "data");
    assert_eq!(input.name(), "test_input");
}

#[test]
fn test_filter_trait_basic() {
    struct TestFilter;
    
    impl Filter for TestFilter {
        fn process(&self, event: Event) -> Result<Event, PluginError> {
            let mut event = event;
            event.set("processed", json!(true));
            Ok(event)
        }
        
        fn name(&self) -> &str {
            "test_filter"
        }
        
        fn config(&self) -> &HashMap<String, serde_json::Value> {
            &HashMap::new()
        }
    }
    
    let filter = TestFilter;
    let event = Event::new(json!({}));
    let processed = filter.process(event).unwrap();
    
    assert_eq!(processed.get("processed").unwrap(), true);
    assert_eq!(filter.name(), "test_filter");
}

#[test]
fn test_output_trait_basic() {
    struct TestOutput {
        written: std::sync::Mutex<Vec<Event>>,
    }
    
    impl Output for TestOutput {
        fn write(&self, event: Event) -> Result<(), PluginError> {
            self.written.lock().unwrap().push(event);
            Ok(())
        }
        
        fn name(&self) -> &str {
            "test_output"
        }
        
        fn config(&self) -> &HashMap<String, serde_json::Value> {
            &HashMap::new()
        }
        
        fn flush(&self) -> Result<(), PluginError> {
            Ok(())
        }
    }
    
    let output = TestOutput {
        written: std::sync::Mutex::new(Vec::new()),
    };
    
    let event = Event::new(json!({"data": "test"}));
    output.write(event).unwrap();
    
    let written = output.written.lock().unwrap();
    assert_eq!(written.len(), 1);
    assert_eq!(written[0].get("data").unwrap(), "test");
    assert_eq!(output.name(), "test_output");
}

#[test]
fn test_plugin_registry() {
    let mut registry = PluginRegistry::new();
    
    // Register input plugin
    registry.register_input("stdin", || {
        Ok(Box::new(TestInput) as Box<dyn Input>)
    });
    
    // Register filter plugin
    registry.register_filter("add_field", || {
        Ok(Box::new(TestFilter) as Box<dyn Filter>)
    });
    
    // Register output plugin
    registry.register_output("stdout", || {
        Ok(Box::new(TestOutput {
            written: std::sync::Mutex::new(Vec::new()),
        }) as Box<dyn Output>)
    });
    
    // Check registrations
    assert!(registry.has_input("stdin"));
    assert!(registry.has_filter("add_field"));
    assert!(registry.has_output("stdout"));
    
    assert!(!registry.has_input("unknown"));
    assert!(!registry.has_filter("unknown"));
    assert!(!registry.has_output("unknown"));
}

#[test]
fn test_plugin_manager() {
    let mut manager = PluginManager::new();
    
    // Create plugin configurations
    let input_config = PluginConfig {
        name: "stdin".to_string(),
        plugin_type: PluginType::Input,
        config: HashMap::new(),
    };
    
    let filter_config = PluginConfig {
        name: "add_field".to_string(),
        plugin_type: PluginType::Filter,
        config: {
            let mut map = HashMap::new();
            map.insert("field".to_string(), json!("host"));
            map.insert("value".to_string(), json!("localhost"));
            map
        },
    };
    
    let output_config = PluginConfig {
        name: "stdout".to_string(),
        plugin_type: PluginType::Output,
        config: HashMap::new(),
    };
    
    // Add plugins to manager
    manager.add_plugin(input_config).unwrap();
    manager.add_plugin(filter_config).unwrap();
    manager.add_plugin(output_config).unwrap();
    
    // Check plugin counts
    assert_eq!(manager.input_count(), 1);
    assert_eq!(manager.filter_count(), 1);
    assert_eq!(manager.output_count(), 1);
    
    // Get plugins
    let inputs = manager.inputs();
    let filters = manager.filters();
    let outputs = manager.outputs();
    
    assert_eq!(inputs.len(), 1);
    assert_eq!(filters.len(), 1);
    assert_eq!(outputs.len(), 1);
}

#[test]
fn test_plugin_factory() {
    let factory = PluginFactory::new();
    
    // Create input plugin from config
    let input_config = PluginConfig {
        name: "stdin".to_string(),
        plugin_type: PluginType::Input,
        config: HashMap::new(),
    };
    
    let input = factory.create_input(&input_config).unwrap();
    assert_eq!(input.name(), "stdin");
    
    // Create filter plugin from config
    let filter_config = PluginConfig {
        name: "add_field".to_string(),
        plugin_type: PluginType::Filter,
        config: {
            let mut map = HashMap::new();
            map.insert("field".to_string(), json!("timestamp"));
            map.insert("value".to_string(), json!("now"));
            map
        },
    };
    
    let filter = factory.create_filter(&filter_config).unwrap();
    assert_eq!(filter.name(), "add_field");
    
    // Create output plugin from config
    let output_config = PluginConfig {
        name: "stdout".to_string(),
        plugin_type: PluginType::Output,
        config: HashMap::new(),
    };
    
    let output = factory.create_output(&output_config).unwrap();
    assert_eq!(output.name(), "stdout");
}

#[test]
fn test_plugin_error_handling() {
    let mut registry = PluginRegistry::new();
    
    // Register a plugin that fails to create
    registry.register_input("failing", || {
        Err(PluginError::InitializationFailed(
            "Test failure".to_string()
        ))
    });
    
    // Should fail when creating the plugin
    let result = registry.create_input("failing");
    assert!(result.is_err());
    
    // Unknown plugin should also fail
    let result = registry.create_input("unknown");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), PluginError::NotFound(_)));
}

#[test]
fn test_plugin_config_validation() {
    let factory = PluginFactory::new();
    
    // Missing required config
    let invalid_config = PluginConfig {
        name: "".to_string(), // Empty name
        plugin_type: PluginType::Input,
        config: HashMap::new(),
    };
    
    let result = factory.create_input(&invalid_config);
    assert!(result.is_err());
    
    // Valid config
    let valid_config = PluginConfig {
        name: "valid".to_string(),
        plugin_type: PluginType::Input,
        config: HashMap::new(),
    };
    
    let result = factory.create_input(&valid_config);
    assert!(result.is_ok());
}

#[test]
fn test_plugin_lifecycle() {
    let mut manager = PluginManager::new();
    
    // Add plugin
    let config = PluginConfig {
        name: "test".to_string(),
        plugin_type: PluginType::Input,
        config: HashMap::new(),
    };
    
    manager.add_plugin(config).unwrap();
    assert_eq!(manager.input_count(), 1);
    
    // Remove plugin
    manager.remove_plugin("test", PluginType::Input).unwrap();
    assert_eq!(manager.input_count(), 0);
    
    // Remove non-existent plugin should fail
    let result = manager.remove_plugin("nonexistent", PluginType::Input);
    assert!(result.is_err());
}

// Test implementations for the tests above
struct TestInput;
impl Input for TestInput {
    fn read(&mut self) -> Result<Option<Event>, PluginError> {
        Ok(Some(Event::new(json!({"test": "data"}))))
    }
    
    fn name(&self) -> &str { "test_input" }
    fn config(&self) -> &HashMap<String, serde_json::Value> { &HashMap::new() }
}

struct TestFilter;
impl Filter for TestFilter {
    fn process(&self, event: Event) -> Result<Event, PluginError> {
        let mut event = event;
        event.set("processed", json!(true));
        Ok(event)
    }
    
    fn name(&self) -> &str { "test_filter" }
    fn config(&self) -> &HashMap<String, serde_json::Value> { &HashMap::new() }
}

struct TestOutput {
    written: std::sync::Mutex<Vec<Event>>,
}
impl Output for TestOutput {
    fn write(&self, event: Event) -> Result<(), PluginError> {
        self.written.lock().unwrap().push(event);
        Ok(())
    }
    
    fn name(&self) -> &str { "test_output" }
    fn config(&self) -> &HashMap<String, serde_json::Value> { &HashMap::new() }
    fn flush(&self) -> Result<(), PluginError> { Ok(()) }
}