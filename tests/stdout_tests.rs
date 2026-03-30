//! Tests for StdoutOutput plugin

use gohangout_rs::event::Event;
use gohangout_rs::output::StdoutOutput;
use gohangout_rs::plugin::{Plugin, PluginConfig, PluginType};
use gohangout_rs::prelude::{Output, PluginResult};
use serde_json::json;
use std::collections::HashMap;

/// Test creating StdoutOutput with default configuration
#[test]
fn test_stdout_output_default() {
    let config = PluginConfig {
        name: "test_stdout".to_string(),
        config: HashMap::new(),
    };
    
    let plugin = StdoutOutput::from_config(&config).unwrap();
    
    // Verify plugin properties
    assert_eq!(plugin.name(), "test_stdout");
    assert_eq!(plugin.plugin_type(), PluginType::Output);
    
    // Validate configuration
    let validation_result = plugin.validate_config();
    assert!(validation_result.is_ok());
    
    // Initialize plugin
    let mut plugin = plugin;
    plugin.initialize().unwrap();
    
    // Create a test event
    let mut data = HashMap::new();
    data.insert("message".to_string(), json!("Test message"));
    data.insert("level".to_string(), json!("INFO"));
    let event = Event::new(Value::Object(data));
    
    // Test writing (this will actually print to stdout during tests)
    let write_result = plugin.write(event);
    assert!(write_result.is_ok());
    
    // Test flush
    let flush_result = plugin.flush();
    assert!(flush_result.is_ok());
    
    // Shutdown plugin
    plugin.shutdown().unwrap();
}

/// Test StdoutOutput with JSON format
#[test]
fn test_stdout_output_json_format() {
    let mut config_map = HashMap::new();
    config_map.insert("format".to_string(), json!("json"));
    config_map.insert("pretty".to_string(), json!(false));
    
    let config = PluginConfig {
        name: "test_json".to_string(),
        config: config_map,
    };
    
    let plugin = StdoutOutput::from_config(&config).unwrap();
    assert_eq!(plugin.name(), "test_json");
    
    // Validate configuration
    assert!(plugin.validate_config().is_ok());
}

/// Test StdoutOutput with pretty format
#[test]
fn test_stdout_output_pretty_format() {
    let mut config_map = HashMap::new();
    config_map.insert("format".to_string(), json!("pretty"));
    config_map.insert("color".to_string(), json!(false)); // Disable color for tests
    
    let config = PluginConfig {
        name: "test_pretty".to_string(),
        config: config_map,
    };
    
    let plugin = StdoutOutput::from_config(&config).unwrap();
    assert_eq!(plugin.name(), "test_pretty");
    
    // Validate configuration
    assert!(plugin.validate_config().is_ok());
}

/// Test StdoutOutput with plain format
#[test]
fn test_stdout_output_plain_format() {
    let mut config_map = HashMap::new();
    config_map.insert("format".to_string(), json!("plain"));
    config_map.insert("timestamp".to_string(), json!(true));
    
    let config = PluginConfig {
        name: "test_plain".to_string(),
        config: config_map,
    };
    
    let plugin = StdoutOutput::from_config(&config).unwrap();
    assert_eq!(plugin.name(), "test_plain");
    
    // Validate configuration
    assert!(plugin.validate_config().is_ok());
}

/// Test StdoutOutput with custom buffer size
#[test]
fn test_stdout_output_buffer_config() {
    let mut config_map = HashMap::new();
    config_map.insert("buffer_size".to_string(), json!(4096));
    config_map.insert("flush_interval".to_string(), json!(500));
    
    let config = PluginConfig {
        name: "test_buffer".to_string(),
        config: config_map,
    };
    
    let plugin = StdoutOutput::from_config(&config).unwrap();
    
    // Validate configuration
    assert!(plugin.validate_config().is_ok());
}

/// Test StdoutOutput configuration validation errors
#[test]
fn test_stdout_output_validation_errors() {
    // Test invalid buffer size (0)
    let mut config_map = HashMap::new();
    config_map.insert("buffer_size".to_string(), json!(0));
    
    let config = PluginConfig {
        name: "test_invalid".to_string(),
        config: config_map,
    };
    
    let plugin = StdoutOutput::from_config(&config).unwrap();
    let validation_result = plugin.validate_config();
    assert!(validation_result.is_err());
    assert!(validation_result.unwrap_err().to_string().contains("Buffer size"));
    
    // Test invalid flush interval (0)
    let mut config_map2 = HashMap::new();
    config_map2.insert("flush_interval".to_string(), json!(0));
    
    let config2 = PluginConfig {
        name: "test_invalid2".to_string(),
        config: config_map2,
    };
    
    let plugin2 = StdoutOutput::from_config(&config2).unwrap();
    let validation_result2 = plugin2.validate_config();
    assert!(validation_result2.is_err());
    assert!(validation_result2.unwrap_err().to_string().contains("Flush interval"));
    
    // Test invalid format
    let mut config_map3 = HashMap::new();
    config_map3.insert("format".to_string(), json!("invalid_format"));
    
    let config3 = PluginConfig {
        name: "test_invalid3".to_string(),
        config: config_map3,
    };
    
    let plugin_result = StdoutOutput::from_config(&config3);
    assert!(plugin_result.is_err());
    assert!(plugin_result.unwrap_err().to_string().contains("Invalid output format"));
}

/// Test StdoutOutput batch writing
#[test]
fn test_stdout_output_batch_write() {
    let config = PluginConfig {
        name: "test_batch".to_string(),
        config: HashMap::new(),
    };
    
    let mut plugin = StdoutOutput::from_config(&config).unwrap();
    plugin.initialize().unwrap();
    
    // Create multiple test events
    let mut events = Vec::new();
    for i in 0..5 {
        let mut data = HashMap::new();
        data.insert("id".to_string(), json!(i));
        data.insert("message".to_string(), json!(format!("Message {}", i)));
        events.push(Event::new(Value::Object(data)));
    }
    
    // Test batch writing
    let write_result = plugin.write_batch(events);
    assert!(write_result.is_ok());
    
    // Test flush
    let flush_result = plugin.flush();
    assert!(flush_result.is_ok());
    
    plugin.shutdown().unwrap();
}

/// Test StdoutOutput statistics
#[test]
fn test_stdout_output_stats() {
    let config = PluginConfig {
        name: "test_stats".to_string(),
        config: HashMap::new(),
    };
    
    let mut plugin = StdoutOutput::from_config(&config).unwrap();
    plugin.initialize().unwrap();
    
    // Initial stats
    let initial_stats = plugin.stats();
    assert_eq!(initial_stats.events_processed, 0);
    
    // Write some events
    let mut data = HashMap::new();
    data.insert("test".to_string(), json!("value"));
    let event = Event::new(Value::Object(data));
    
    plugin.write(event.clone()).unwrap();
    plugin.write(event.clone()).unwrap();
    
    // Check updated stats
    let updated_stats = plugin.stats();
    assert_eq!(updated_stats.events_processed, 2);
    assert!(updated_stats.last_event_time.is_some());
    assert!(updated_stats.duration > std::time::Duration::from_secs(0));
    
    plugin.shutdown().unwrap();
    
    // Final stats after shutdown
    let final_stats = plugin.stats();
    assert_eq!(final_stats.events_processed, 2);
}

/// Test StdoutOutput plugin registration
#[test]
fn test_stdout_output_registration() {
    use gohangout_rs::output;
    
    // Create factory and register plugins
    let mut factory = output::default_factory();
    
    // Check that stdout plugin is registered
    assert!(factory.supports_plugin("stdout", gohangout_rs::plugin::PluginType::Output));
    
    // Create stdout plugin instance
    let mut config_map = HashMap::new();
    config_map.insert("format".to_string(), json!("json"));
    
    let config = PluginConfig {
        name: "registered_stdout".to_string(),
        config: config_map,
    };
    
    let plugin = factory.create_output("stdout", &config);
    assert!(plugin.is_ok());
    
    let mut plugin = plugin.unwrap();
    plugin.initialize().unwrap();
    
    // Should be able to write
    let mut data = HashMap::new();
    data.insert("test".to_string(), json!("registered"));
    let event = Event::new(Value::Object(data));
    
    let write_result = plugin.write(event);
    assert!(write_result.is_ok());
    
    plugin.shutdown().unwrap();
}

/// Test StdoutOutput with different event types
#[test]
fn test_stdout_output_various_events() {
    let config = PluginConfig {
        name: "test_various".to_string(),
        config: HashMap::new(),
    };
    
    let mut plugin = StdoutOutput::from_config(&config).unwrap();
    plugin.initialize().unwrap();
    
    // Test with simple event
    let mut simple_data = HashMap::new();
    simple_data.insert("message".to_string(), json!("Simple message"));
    let simple_event = Event::new(Value::Object(simple_data));
    assert!(plugin.write(simple_event).is_ok());
    
    // Test with complex event (nested JSON)
    let mut complex_data = HashMap::new();
    complex_data.insert("user".to_string(), json!({
        "id": 123,
        "name": "Test User",
        "email": "test@example.com"
    }));
    complex_data.insert("request".to_string(), json!({
        "method": "GET",
        "path": "/api/test",
        "status": 200
    }));
    let complex_event = Event::new(Value::Object(complex_data));
    assert!(plugin.write(complex_event).is_ok());
    
    // Test with array data
    let mut array_data = HashMap::new();
    array_data.insert("tags".to_string(), json!(["rust", "testing", "etl"]));
    array_data.insert("numbers".to_string(), json!([1, 2, 3, 4, 5]));
    let array_event = Event::new(Value::Object(array_data));
    assert!(plugin.write(array_event).is_ok());
    
    plugin.shutdown().unwrap();
}