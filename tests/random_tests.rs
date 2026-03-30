//! Tests for RandomInput plugin

use gohangout_rs::event::Event;
use gohangout_rs::input::RandomInput;
use gohangout_rs::plugin::{Plugin, PluginConfig, PluginType};
use gohangout_rs::prelude::{Input, PluginResult};
use serde_json::json;
use std::collections::HashMap;

/// Test creating RandomInput with default configuration
#[test]
fn test_random_input_default() {
    let config = PluginConfig {
        name: "test_random".to_string(),
        config: HashMap::new(),
    };
    
    let mut plugin = RandomInput::from_config(&config).unwrap();
    
    // Verify plugin properties
    assert_eq!(plugin.name(), "test_random");
    assert_eq!(plugin.plugin_type(), PluginType::Input);
    
    // Initialize plugin
    plugin.initialize().unwrap();
    
    // Read some events
    for _ in 0..5 {
        let event = plugin.read().unwrap();
        assert!(event.is_some());
        let event = event.unwrap();
        
        // Verify event has data
        let data = event.get_data();
        assert!(data.is_object());
        
        // Verify basic fields exist
        let obj = data.as_object().unwrap();
        assert!(obj.contains_key("timestamp"));
        assert!(obj.contains_key("level") || obj.contains_key("extra"));
    }
    
    // Shutdown plugin
    plugin.shutdown().unwrap();
}

/// Test RandomInput with simple mode
#[test]
fn test_random_input_simple_mode() {
    let mut config_map = HashMap::new();
    config_map.insert("mode".to_string(), json!("simple"));
    config_map.insert("rate".to_string(), json!(100)); // 100 events/sec
    config_map.insert("count".to_string(), json!(10)); // Generate 10 events
    
    let config = PluginConfig {
        name: "test_simple".to_string(),
        config: config_map,
    };
    
    let mut plugin = RandomInput::from_config(&config).unwrap();
    plugin.initialize().unwrap();
    
    // Read exactly 10 events
    for i in 0..10 {
        let event = plugin.read().unwrap();
        assert!(event.is_some(), "Event {} should exist", i);
        
        let event = event.unwrap();
        let data = event.get_data().as_object().unwrap();
        
        // Verify simple mode fields
        assert!(data.contains_key("timestamp"));
        assert!(data.contains_key("level"));
        assert!(data.contains_key("message"));
        assert!(data.contains_key("extra"));
    }
    
    // Next read should return None (count limit reached)
    let event = plugin.read().unwrap();
    assert!(event.is_none());
    
    plugin.shutdown().unwrap();
}

/// Test RandomInput with complex mode
#[test]
fn test_random_input_complex_mode() {
    let mut config_map = HashMap::new();
    config_map.insert("mode".to_string(), json!("complex"));
    config_map.insert("count".to_string(), json!(3));
    
    let config = PluginConfig {
        name: "test_complex".to_string(),
        config: config_map,
    };
    
    let mut plugin = RandomInput::from_config(&config).unwrap();
    plugin.initialize().unwrap();
    
    for _ in 0..3 {
        let event = plugin.read().unwrap().unwrap();
        let data = event.get_data().as_object().unwrap();
        
        // Verify complex mode structure
        assert!(data.contains_key("id"));
        assert!(data.contains_key("timestamp"));
        
        // Verify user object
        let user = data.get("user").unwrap().as_object().unwrap();
        assert!(user.contains_key("id"));
        assert!(user.contains_key("name"));
        assert!(user.contains_key("email"));
        assert!(user.contains_key("active"));
        
        // Verify request object
        let request = data.get("request").unwrap().as_object().unwrap();
        assert!(request.contains_key("method"));
        assert!(request.contains_key("path"));
        assert!(request.contains_key("status"));
        assert!(request.contains_key("duration_ms"));
        assert!(request.contains_key("client_ip"));
        
        // Verify metrics object
        let metrics = data.get("metrics").unwrap().as_object().unwrap();
        assert!(metrics.contains_key("cpu_usage"));
        assert!(metrics.contains_key("memory_mb"));
        assert!(metrics.contains_key("queue_length"));
    }
    
    plugin.shutdown().unwrap();
}

/// Test RandomInput with custom mode
#[test]
fn test_random_input_custom_mode() {
    let mut fields_map = HashMap::new();
    fields_map.insert("user_id".to_string(), json!("number"));
    fields_map.insert("username".to_string(), json!("string"));
    fields_map.insert("email".to_string(), json!("email"));
    fields_map.insert("status".to_string(), json!("enum:active,inactive,pending"));
    
    let mut config_map = HashMap::new();
    config_map.insert("mode".to_string(), json!("custom"));
    config_map.insert("fields".to_string(), json!(fields_map));
    config_map.insert("count".to_string(), json!(5));
    
    let config = PluginConfig {
        name: "test_custom".to_string(),
        config: config_map,
    };
    
    let mut plugin = RandomInput::from_config(&config).unwrap();
    plugin.initialize().unwrap();
    
    for _ in 0..5 {
        let event = plugin.read().unwrap().unwrap();
        let data = event.get_data().as_object().unwrap();
        
        // Verify default fields
        assert!(data.contains_key("timestamp"));
        assert!(data.contains_key("id"));
        
        // Verify custom fields
        assert!(data.contains_key("user_id"));
        assert!(data.contains_key("username"));
        assert!(data.contains_key("email"));
        assert!(data.contains_key("status"));
        
        // Verify field types
        let status = data.get("status").unwrap().as_str().unwrap();
        assert!(matches!(status, "active" | "inactive" | "pending"));
        
        let email = data.get("email").unwrap().as_str().unwrap();
        assert!(email.contains('@'));
    }
    
    plugin.shutdown().unwrap();
}

/// Test RandomInput with rate limiting
#[test]
fn test_random_input_rate_limiting() {
    let mut config_map = HashMap::new();
    config_map.insert("mode".to_string(), json!("simple"));
    config_map.insert("rate".to_string(), json!(2)); // 2 events/sec
    config_map.insert("count".to_string(), json!(3));
    
    let config = PluginConfig {
        name: "test_rate".to_string(),
        config: config_map,
    };
    
    let mut plugin = RandomInput::from_config(&config).unwrap();
    plugin.initialize().unwrap();
    
    // Time the generation of 3 events at 2 events/sec
    // Should take at least 1 second (for 2 events) + a bit more for the 3rd
    let start = std::time::Instant::now();
    
    for _ in 0..3 {
        let event = plugin.read().unwrap();
        assert!(event.is_some());
    }
    
    let elapsed = start.elapsed();
    // Should take at least 1 second (2 events at 2/sec = 1 sec between first two)
    // Actually for 3 events at 2/sec: intervals are 0.5s, 0.5s = 1s total minimum
    assert!(elapsed >= std::time::Duration::from_millis(900));
    
    plugin.shutdown().unwrap();
}

/// Test RandomInput with infinite generation
#[test]
fn test_random_input_infinite() {
    let mut config_map = HashMap::new();
    config_map.insert("mode".to_string(), json!("simple"));
    config_map.insert("rate".to_string(), json!(0)); // As fast as possible
    config_map.insert("count".to_string(), json!(0)); // Infinite
    
    let config = PluginConfig {
        name: "test_infinite".to_string(),
        config: config_map,
    };
    
    let mut plugin = RandomInput::from_config(&config).unwrap();
    plugin.initialize().unwrap();
    
    // Should be able to read many events without stopping
    for i in 0..100 {
        let event = plugin.read().unwrap();
        assert!(event.is_some(), "Event {} should exist", i);
        
        // Plugin should still be ready
        assert!(plugin.is_ready());
    }
    
    plugin.shutdown().unwrap();
}

/// Test RandomInput configuration validation
#[test]
fn test_random_input_validation() {
    // Test invalid mode
    let mut config_map = HashMap::new();
    config_map.insert("mode".to_string(), json!("invalid_mode"));
    
    let config = PluginConfig {
        name: "test_invalid".to_string(),
        config: config_map,
    };
    
    let result = RandomInput::from_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid mode"));
    
    // Test invalid field type
    let mut fields_map = HashMap::new();
    fields_map.insert("test".to_string(), json!("invalid_type"));
    
    let mut config_map2 = HashMap::new();
    config_map2.insert("mode".to_string(), json!("custom"));
    config_map2.insert("fields".to_string(), json!(fields_map));
    
    let config2 = PluginConfig {
        name: "test_invalid_field".to_string(),
        config: config_map2,
    };
    
    let result2 = RandomInput::from_config(&config2);
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("Invalid field type"));
    
    // Test valid configuration
    let mut config_map3 = HashMap::new();
    config_map3.insert("mode".to_string(), json!("simple"));
    config_map3.insert("rate".to_string(), json!(10));
    config_map3.insert("count".to_string(), json!(100));
    config_map3.insert("seed".to_string(), json!(12345));
    
    let config3 = PluginConfig {
        name: "test_valid".to_string(),
        config: config_map3,
    };
    
    let plugin = RandomInput::from_config(&config3);
    assert!(plugin.is_ok());
    
    let plugin = plugin.unwrap();
    let validation_result = plugin.validate_config();
    assert!(validation_result.is_ok());
}

/// Test RandomInput statistics
#[test]
fn test_random_input_stats() {
    let mut config_map = HashMap::new();
    config_map.insert("mode".to_string(), json!("simple"));
    config_map.insert("count".to_string(), json!(5));
    
    let config = PluginConfig {
        name: "test_stats".to_string(),
        config: config_map,
    };
    
    let mut plugin = RandomInput::from_config(&config).unwrap();
    plugin.initialize().unwrap();
    
    // Initial stats
    let initial_stats = plugin.stats();
    assert_eq!(initial_stats.events_processed, 0);
    
    // Generate some events
    for i in 0..5 {
        let event = plugin.read().unwrap();
        assert!(event.is_some(), "Event {} should exist", i);
        
        let stats = plugin.stats();
        assert_eq!(stats.events_processed, i + 1);
        assert!(stats.last_event_time.is_some());
    }
    
    // Final stats after shutdown
    plugin.shutdown().unwrap();
    let final_stats = plugin.stats();
    assert_eq!(final_stats.events_processed, 5);
    assert!(final_stats.duration > std::time::Duration::from_secs(0));
}

/// Test RandomInput plugin registration
#[test]
fn test_random_input_registration() {
    use gohangout_rs::input;
    
    // Create factory and register plugins
    let mut factory = input::default_factory();
    
    // Check that random plugin is registered
    assert!(factory.supports_plugin("random", gohangout_rs::plugin::PluginType::Input));
    
    // Create random plugin instance
    let mut config_map = HashMap::new();
    config_map.insert("mode".to_string(), json!("simple"));
    config_map.insert("count".to_string(), json!(3));
    
    let config = PluginConfig {
        name: "registered_random".to_string(),
        config: config_map,
    };
    
    let plugin = factory.create_input("random", &config);
    assert!(plugin.is_ok());
    
    let mut plugin = plugin.unwrap();
    plugin.initialize().unwrap();
    
    // Should be able to read events
    for _ in 0..3 {
        let event = plugin.read().unwrap();
        assert!(event.is_some());
    }
    
    plugin.shutdown().unwrap();
}