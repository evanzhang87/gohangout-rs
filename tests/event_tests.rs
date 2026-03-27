//! Event model tests

use gohangout_rs::event::*;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_event_creation() {
    let event = Event::new(json!({"message": "test log"}));
    
    assert!(!event.id().is_nil());
    assert!(event.timestamp() <= chrono::Utc::now());
    assert_eq!(event.get("message").unwrap(), "test log");
}

#[test]
fn test_event_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("host".to_string(), json!("localhost"));
    metadata.insert("service".to_string(), json!("web"));
    
    let event = Event::with_metadata(json!({"message": "test"}), metadata);
    
    assert_eq!(event.metadata().get("host").unwrap(), "localhost");
    assert_eq!(event.metadata().get("service").unwrap(), "web");
    assert_eq!(event.get("message").unwrap(), "test");
}

#[test]
fn test_event_set_and_get() {
    let mut event = Event::new(json!({}));
    
    // Set field
    event.set("message", json!("hello world"));
    event.set("level", json!("info"));
    
    // Get field
    assert_eq!(event.get("message").unwrap(), "hello world");
    assert_eq!(event.get("level").unwrap(), "info");
    
    // Get non-existent field
    assert!(event.get("nonexistent").is_none());
}

#[test]
fn test_event_serialization() {
    let original = Event::new(json!({
        "message": "test log",
        "level": "error",
        "timestamp": "2024-01-01T00:00:00Z"
    }));
    
    // Serialize to JSON
    let serialized = original.to_json().expect("Failed to serialize");
    
    // Deserialize back
    let deserialized = Event::from_json(&serialized).expect("Failed to deserialize");
    
    // Should preserve data
    assert_eq!(deserialized.get("message").unwrap(), "test log");
    assert_eq!(deserialized.get("level").unwrap(), "error");
    
    // ID should be preserved
    assert_eq!(deserialized.id(), original.id());
}

#[test]
fn test_event_clone() {
    let event1 = Event::new(json!({"test": "data"}));
    let event2 = event1.clone();
    
    // Should have same data
    assert_eq!(event1.get("test").unwrap(), event2.get("test").unwrap());
    
    // But different IDs (clones get new ID)
    assert_ne!(event1.id(), event2.id());
}

#[test]
fn test_event_remove_field() {
    let mut event = Event::new(json!({
        "keep": "this",
        "remove": "this"
    }));
    
    // Remove field
    let removed = event.remove("remove");
    assert_eq!(removed.unwrap(), "this");
    
    // Field should be gone
    assert!(event.get("remove").is_none());
    assert_eq!(event.get("keep").unwrap(), "this");
    
    // Remove non-existent field
    assert!(event.remove("nonexistent").is_none());
}

#[test]
fn test_event_metadata_operations() {
    let mut event = Event::new(json!({}));
    
    // Add metadata
    event.add_metadata("host", json!("server1"));
    event.add_metadata("environment", json!("production"));
    
    // Get metadata
    assert_eq!(event.get_metadata("host").unwrap(), "server1");
    assert_eq!(event.get_metadata("environment").unwrap(), "production");
    
    // Remove metadata
    let removed = event.remove_metadata("host");
    assert_eq!(removed.unwrap(), "server1");
    assert!(event.get_metadata("host").is_none());
}

#[test]
fn test_event_timestamp_operations() {
    let now = chrono::Utc::now();
    let event = Event::new(json!({}));
    
    // Event timestamp should be close to now
    let diff = event.timestamp().signed_duration_since(now);
    assert!(diff.num_seconds().abs() <= 1);
    
    // Test with custom timestamp
    let custom_time = chrono::Utc::now() - chrono::Duration::hours(1);
    let mut event2 = Event::new(json!({}));
    event2.set_timestamp(custom_time);
    
    assert_eq!(event2.timestamp(), custom_time);
}

#[test]
fn test_event_merge() {
    let mut event1 = Event::new(json!({
        "field1": "value1",
        "common": "old"
    }));
    
    let event2 = Event::new(json!({
        "field2": "value2",
        "common": "new"
    }));
    
    // Merge event2 into event1
    event1.merge(event2);
    
    // Should have all fields
    assert_eq!(event1.get("field1").unwrap(), "value1");
    assert_eq!(event1.get("field2").unwrap(), "value2");
    
    // Common field should be overwritten
    assert_eq!(event1.get("common").unwrap(), "new");
}

#[test]
fn test_event_error_handling() {
    // Invalid JSON should fail
    let result = Event::from_json("invalid json");
    assert!(result.is_err());
    
    // Empty JSON should work
    let event = Event::from_json("{}").expect("Should parse empty JSON");
    assert!(event.data().is_object());
}

#[test]
fn test_pipeline_basic() {
    let pipeline = SimplePipeline::new();
    
    // Create test events
    let events = vec![
        Event::new(json!({"id": 1})),
        Event::new(json!({"id": 2})),
        Event::new(json!({"id": 3})),
    ];
    
    // Process events
    let processed = pipeline.process(events).expect("Pipeline should process");
    
    // Should process all events
    assert_eq!(processed.len(), 3);
}

#[test]
fn test_pipeline_with_processor() {
    let mut pipeline = SimplePipeline::new();
    
    // Add a processor that adds a field
    pipeline.add_processor(Box::new(|mut event| {
        event.set("processed", json!(true));
        Ok(event)
    }));
    
    let mut event = Event::new(json!({}));
    event = pipeline.process_event(event).expect("Should process");
    
    assert_eq!(event.get("processed").unwrap(), true);
}

#[test]
fn test_pipeline_error_propagation() {
    let mut pipeline = SimplePipeline::new();
    
    // Add a processor that fails
    pipeline.add_processor(Box::new(|_| {
        Err("Test error".into())
    }));
    
    let event = Event::new(json!({}));
    let result = pipeline.process_event(event);
    
    assert!(result.is_err());
}