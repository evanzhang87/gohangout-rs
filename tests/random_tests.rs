//! Tests for RandomInput plugin (simplified version matching GoHangout)

use gohangout_rs::event::Event;
use gohangout_rs::input::RandomInput;
use gohangout_rs::plugin::{Plugin, PluginConfig, PluginType, PluginResult};
use gohangout_rs::prelude::Input;
use serde_json::json;
use std::collections::HashMap;

/// Test creating RandomInput with required configuration
#[test]
fn test_random_input_basic() {
    let mut config_map = HashMap::new();
    config_map.insert("from".to_string(), json!(1));
    config_map.insert("to".to_string(), json!(100));
    
    let mut config = PluginConfig::new("test_random", PluginType::Input);
    config.config = config_map;
    
    let mut plugin = RandomInput::from_config(&config).unwrap();
    
    // Verify plugin properties
    assert_eq!(plugin.name(), "test_random");
    assert_eq!(plugin.plugin_type(), PluginType::Input);
    
    // Validate configuration
    let validation_result = plugin.validate_config();
    assert!(validation_result.is_ok());
    
    // Initialize plugin
    plugin.initialize().unwrap();
    
    // Read some events and verify they contain random numbers in range
    for _ in 0..10 {
        let event_result = plugin.read();
        assert!(event_result.is_ok());
        
        let event_opt = event_result.unwrap();
        assert!(event_opt.is_some());
        
        let event = event_opt.unwrap();
        let data = event.data();
        
        // Verify event has message field with a number string
        let message = data.get("message")
            .expect("Event should have 'message' field");
        
        assert!(message.is_string());
        let message_str = message.as_str().unwrap();
        
        // Parse the number and verify it's in range
        let num: i64 = message_str.parse()
            .expect("Message should be a valid integer");
        
        assert!(num >= 1 && num <= 100, "Number {} should be in range [1, 100]", num);
    }
    
    // Shutdown plugin
    plugin.shutdown().unwrap();
}

/// Test RandomInput with max_messages limit
#[test]
fn test_random_input_max_messages() {
    let mut config_map = HashMap::new();
    config_map.insert("from".to_string(), json!(1));
    config_map.insert("to".to_string(), json!(10));
    config_map.insert("max_messages".to_string(), json!(5));
    
    let mut config = PluginConfig::new("test_limited", PluginType::Input);
    config.config = config_map;
    
    let mut plugin = RandomInput::from_config(&config).unwrap();
    plugin.initialize().unwrap();
    
    // Read exactly 5 events
    for i in 0..5 {
        let event_opt = plugin.read().unwrap();
        assert!(event_opt.is_some(), "Should get event {} of 5", i + 1);
        
        let event = event_opt.unwrap();
        let data = event.data();
        let message = data.get("message").unwrap().as_str().unwrap();
        let num: i64 = message.parse().unwrap();
        assert!(num >= 1 && num <= 10);
    }
    
    // Next read should return None (max messages reached)
    let event_opt = plugin.read().unwrap();
    assert!(event_opt.is_none(), "Should return None after max_messages");
    
    // Status should be Stopped
    assert_eq!(plugin.status(), gohangout_rs::plugin::traits::PluginStatus::Stopped);
    
    plugin.shutdown().unwrap();
}

/// Test RandomInput with unlimited messages (max_messages = -1)
#[test]
fn test_random_input_unlimited() {
    let mut config_map = HashMap::new();
    config_map.insert("from".to_string(), json!(1));
    config_map.insert("to".to_string(), json!(5));
    config_map.insert("max_messages".to_string(), json!(-1));
    
    let mut config = PluginConfig::new("test_unlimited", PluginType::Input);
    config.config = config_map;
    
    let mut plugin = RandomInput::from_config(&config).unwrap();
    plugin.initialize().unwrap();
    
    // Read many events (more than a small number)
    for i in 0..20 {
        let event_opt = plugin.read().unwrap();
        assert!(event_opt.is_some(), "Should get event {} (unlimited)", i + 1);
    }
    
    // Status should still be Ready (unlimited)
    assert_eq!(plugin.status(), gohangout_rs::plugin::traits::PluginStatus::Ready);
    
    plugin.shutdown().unwrap();
}

/// Test RandomInput configuration validation errors
#[test]
fn test_random_input_validation_errors() {
    // Missing 'from' parameter
    let mut config_map = HashMap::new();
    config_map.insert("to".to_string(), json!(100));
    
    let mut config = PluginConfig::new("test_missing_from", PluginType::Input);
    config.config = config_map;
    
    let result = RandomInput::from_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("from must be configured"));
    
    // Missing 'to' parameter
    let mut config_map2 = HashMap::new();
    config_map2.insert("from".to_string(), json!(1));
    
    let mut config2 = PluginConfig::new("test_missing_to", PluginType::Input);
    config2.config = config_map2;
    
    let result2 = RandomInput::from_config(&config2);
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("to must be configured"));
    
    // Invalid range (from > to)
    let mut config_map3 = HashMap::new();
    config_map3.insert("from".to_string(), json!(100));
    config_map3.insert("to".to_string(), json!(1));
    
    let mut config3 = PluginConfig::new("test_invalid_range", PluginType::Input);
    config3.config = config_map3;
    
    let result3 = RandomInput::from_config(&config3);
    assert!(result3.is_err());
    assert!(result3.unwrap_err().to_string().contains("must be less than or equal"));
}

/// Test RandomInput statistics
#[test]
fn test_random_input_stats() {
    let mut config_map = HashMap::new();
    config_map.insert("from".to_string(), json!(1));
    config_map.insert("to".to_string(), json!(100));
    
    let mut config = PluginConfig::new("test_stats", PluginType::Input);
    config.config = config_map;
    
    let mut plugin = RandomInput::from_config(&config).unwrap();
    plugin.initialize().unwrap();
    
    // Initial stats
    let initial_stats = plugin.stats();
    assert_eq!(initial_stats.events_read, 0);
    
    // Read some events
    for _ in 0..7 {
        plugin.read().unwrap();
    }
    
    // Check updated stats
    let updated_stats = plugin.stats();
    assert_eq!(updated_stats.events_read, 7);
    // Note: last_event_time is not tracked in InputStats
    
    plugin.shutdown().unwrap();
}

/// Test RandomInput default configuration
#[test]
fn test_random_input_default_config() {
    let plugin = RandomInput::default();
    
    assert_eq!(plugin.name(), "random");
    assert_eq!(plugin.plugin_type(), PluginType::Input);
    
    let config = plugin.config();
    assert_eq!(config.get("from").unwrap().as_i64().unwrap(), 1);
    assert_eq!(config.get("to").unwrap().as_i64().unwrap(), 100);
    
    // Default should not have max_messages
    assert!(config.get("max_messages").is_none());
}