//! Event structure for GoHangout-rs

use chrono::{DateTime, Utc, FixedOffset};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Core event structure for ETL processing
#[derive(Debug, Clone)]
pub struct Event {
    /// Unique identifier for the event
    id: Uuid,
    
    /// Event timestamp (when the event was created/ingested)
    timestamp: DateTime<Utc>,
    
    /// Event metadata (tags, routing information, etc.)
    metadata: HashMap<String, Value>,
    
    /// Event data (the actual payload)
    data: Value,
}

impl Event {
    /// Create a new event with the given data
    pub fn new(data: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            data,
        }
    }
    
    /// Create a new event with metadata
    pub fn with_metadata(data: Value, metadata: HashMap<String, Value>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            metadata,
            data,
        }
    }
    
    /// Get the event ID
    pub fn id(&self) -> Uuid {
        self.id
    }
    
    /// Get the event timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
    
    /// Set the event timestamp
    pub fn set_timestamp(&mut self, timestamp: DateTime<Utc>) {
        self.timestamp = timestamp;
    }
    
    /// Get the event data
    pub fn data(&self) -> &Value {
        &self.data
    }
    
    /// Get mutable access to event data
    pub fn data_mut(&mut self) -> &mut Value {
        &mut self.data
    }
    
    /// Get the event metadata
    pub fn metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }
    
    /// Get mutable access to event metadata
    pub fn metadata_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.metadata
    }
    
    /// Get a field from the event data
    pub fn get(&self, field: &str) -> Option<&Value> {
        self.data.get(field)
    }
    
    /// Set a field in the event data
    pub fn set(&mut self, field: &str, value: Value) {
        if let Some(obj) = self.data.as_object_mut() {
            obj.insert(field.to_string(), value);
        } else {
            // If data is not an object, convert it to one
            let mut map = HashMap::new();
            map.insert(field.to_string(), value);
            self.data = Value::Object(map.into_iter().collect());
        }
    }
    
    /// Remove a field from the event data
    pub fn remove(&mut self, field: &str) -> Option<Value> {
        self.data.as_object_mut()
            .and_then(|obj| obj.remove(field))
    }
    
    /// Check if the event data contains a field
    pub fn contains(&self, field: &str) -> bool {
        self.data.get(field).is_some()
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }
    
    /// Set metadata value
    pub fn add_metadata(&mut self, key: &str, value: Value) {
        self.metadata.insert(key.to_string(), value);
    }
    
    /// Remove metadata value
    pub fn remove_metadata(&mut self, key: &str) -> Option<Value> {
        self.metadata.remove(key)
    }
    
    /// Merge another event into this one
    pub fn merge(&mut self, other: Event) {
        // Merge metadata
        for (key, value) in other.metadata {
            self.metadata.insert(key, value);
        }
        
        // Merge data
        if let (Some(self_obj), Some(other_obj)) = (
            self.data.as_object_mut(),
            other.data.as_object(),
        ) {
            for (key, value) in other_obj {
                self_obj.insert(key.clone(), value.clone());
            }
        }
    }
    
    /// Ensure the event data contains @timestamp field
    /// If the event already has @timestamp, keep it
    /// If not, add @timestamp with current time in the specified format
    pub fn ensure_timestamp(&mut self) {
        if !self.contains("@timestamp") {
            // Get current time in Asia/Shanghai timezone (+08:00)
            let utc_now = Utc::now();
            let shanghai_offset = FixedOffset::east_opt(8 * 3600).unwrap();
            let shanghai_time = utc_now.with_timezone(&shanghai_offset);
            
            // Format with nanosecond precision
            let timestamp_str = format!("{}", shanghai_time.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true));
            
            self.set("@timestamp", Value::String(timestamp_str));
        }
    }
    
    /// Convert event to JSON string
    pub fn to_json(&self) -> serde_json::Result<String> {
        let event_json = serde_json::json!({
            "id": self.id,
            "timestamp": self.timestamp.to_rfc3339(),
            "metadata": self.metadata,
            "data": self.data,
        });
        
        serde_json::to_string(&event_json)
    }
    
    /// Create event from JSON string
    pub fn from_json(json_str: &str) -> serde_json::Result<Self> {
        let value: Value = serde_json::from_str(json_str)?;
        
        let id = value.get("id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::new_v4);
        
        let timestamp = value.get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        
        let metadata = value.get("metadata")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();
        
        let data = value.get("data")
            .cloned()
            .unwrap_or(Value::Null);
        
        Ok(Self {
            id,
            timestamp,
            metadata,
            data,
        })
    }
    
    /// Check if the event is empty (has no data)
    pub fn is_empty(&self) -> bool {
        self.data.is_null() || (self.data.is_object() && self.data.as_object().unwrap().is_empty())
    }
    
    /// Get the size of the event in bytes (approximate)
    pub fn size(&self) -> usize {
        // Approximate size calculation
        let metadata_size: usize = self.metadata.iter()
            .map(|(k, v)| k.len() + v.to_string().len())
            .sum();
        
        let data_size = self.data.to_string().len();
        
        metadata_size + data_size + 36 // UUID size
    }
}

impl Default for Event {
    fn default() -> Self {
        Self::new(Value::Object(Default::default()))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_event_basics() {
        let event = Event::new(json!({"message": "test"}));
        
        assert!(!event.id().is_nil());
        assert!(event.timestamp() <= Utc::now());
        assert_eq!(event.get("message").unwrap(), "test");
    }
    
    #[test]
    fn test_event_serialization() {
        let original = Event::new(json!({
            "message": "hello",
            "level": "info"
        }));
        
        let json_str = original.to_json().unwrap();
        let restored = Event::from_json(&json_str).unwrap();
        
        assert_eq!(restored.id(), original.id());
        assert_eq!(restored.get("message").unwrap(), "hello");
        assert_eq!(restored.get("level").unwrap(), "info");
    }
    
    #[test]
    fn test_event_merge() {
        let mut event1 = Event::new(json!({"a": 1}));
        event1.add_metadata("host", json!("server1"));
        
        let event2 = Event::new(json!({"b": 2}));
        
        event1.merge(event2);
        
        assert_eq!(event1.get("a").unwrap(), 1);
        assert_eq!(event1.get("b").unwrap(), 2);
        assert_eq!(event1.get_metadata("host").unwrap(), "server1");
    }
    
    #[test]
    fn test_ensure_timestamp() {
        // Test 1: Event without @timestamp
        let mut event1 = Event::new(json!({"message": "test"}));
        assert!(!event1.contains("@timestamp"));
        
        event1.ensure_timestamp();
        assert!(event1.contains("@timestamp"));
        
        let timestamp = event1.get("@timestamp").unwrap();
        assert!(timestamp.is_string());
        
        // Check format: should match RFC3339 with nanoseconds and timezone
        let timestamp_str = timestamp.as_str().unwrap();
        assert!(timestamp_str.contains("T"));
        assert!(timestamp_str.contains("+08:00")); // Shanghai timezone
        
        // Test 2: Event with existing @timestamp
        let existing_timestamp = "2026-04-03T11:59:23.464765211+08:00";
        let mut event2 = Event::new(json!({
            "message": "test",
            "@timestamp": existing_timestamp
        }));
        
        event2.ensure_timestamp();
        assert_eq!(event2.get("@timestamp").unwrap(), existing_timestamp);
    }
}