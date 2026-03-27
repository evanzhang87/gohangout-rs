//! Processor implementations
#![allow(dead_code)]

use crate::event::{Event, ProcessorTrait};
use std::error::Error;

/// Simple processor that applies a transformation to events
pub struct SimpleProcessor {
    name: String,
    description: String,
}

impl SimpleProcessor {
    /// Create a new simple processor
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
        }
    }
    
    /// Get processor description
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl ProcessorTrait for SimpleProcessor {
    fn process(&self, event: Event) -> Result<Event, Box<dyn Error>> {
        // Default implementation does nothing
        Ok(event)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Processor that adds a field to events
pub struct AddFieldProcessor {
    name: String,
    field: String,
    value: serde_json::Value,
}

impl AddFieldProcessor {
    /// Create a new add field processor
    pub fn new(field: &str, value: serde_json::Value) -> Self {
        Self {
            name: format!("add_field_{}", field),
            field: field.to_string(),
            value,
        }
    }
    
    /// Get the field name
    pub fn field(&self) -> &str {
        &self.field
    }
    
    /// Get the field value
    pub fn value(&self) -> &serde_json::Value {
        &self.value
    }
}

impl ProcessorTrait for AddFieldProcessor {
    fn process(&self, mut event: Event) -> Result<Event, Box<dyn Error>> {
        event.set(&self.field, self.value.clone());
        Ok(event)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Processor that removes a field from events
pub struct RemoveFieldProcessor {
    name: String,
    field: String,
}

impl RemoveFieldProcessor {
    /// Create a new remove field processor
    pub fn new(field: &str) -> Self {
        Self {
            name: format!("remove_field_{}", field),
            field: field.to_string(),
        }
    }
    
    /// Get the field name
    pub fn field(&self) -> &str {
        &self.field
    }
}

impl ProcessorTrait for RemoveFieldProcessor {
    fn process(&self, mut event: Event) -> Result<Event, Box<dyn Error>> {
        event.remove(&self.field);
        Ok(event)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Processor that renames a field
pub struct RenameFieldProcessor {
    name: String,
    from: String,
    to: String,
}

impl RenameFieldProcessor {
    /// Create a new rename field processor
    pub fn new(from: &str, to: &str) -> Self {
        Self {
            name: format!("rename_{}_to_{}", from, to),
            from: from.to_string(),
            to: to.to_string(),
        }
    }
    
    /// Get source field name
    pub fn from(&self) -> &str {
        &self.from
    }
    
    /// Get target field name
    pub fn to(&self) -> &str {
        &self.to
    }
}

impl ProcessorTrait for RenameFieldProcessor {
    fn process(&self, mut event: Event) -> Result<Event, Box<dyn Error>> {
        if let Some(value) = event.remove(&self.from) {
            event.set(&self.to, value);
        }
        Ok(event)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Processor that filters events based on a condition
pub struct FilterProcessor<F>
where
    F: Fn(&Event) -> bool + Send + Sync,
{
    name: String,
    condition: F,
}

impl<F> FilterProcessor<F>
where
    F: Fn(&Event) -> bool + Send + Sync,
{
    /// Create a new filter processor
    pub fn new(name: &str, condition: F) -> Self {
        Self {
            name: name.to_string(),
            condition,
        }
    }
}

impl<F> ProcessorTrait for FilterProcessor<F>
where
    F: Fn(&Event) -> bool + Send + Sync + 'static,
{
    fn process(&self, event: Event) -> Result<Event, Box<dyn Error>> {
        if (self.condition)(&event) {
            Ok(event)
        } else {
            // Filter out the event by returning an error
            // In a real implementation, we might want a different mechanism
            Err(format!("Event filtered by {}", self.name).into())
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_add_field_processor() {
        let processor = AddFieldProcessor::new("host", json!("localhost"));
        let event = Event::new(json!({"message": "test"}));
        
        let processed = processor.process(event).unwrap();
        assert_eq!(processed.get("host").unwrap(), "localhost");
        assert_eq!(processed.get("message").unwrap(), "test");
        assert_eq!(processor.name(), "add_field_host");
    }
    
    #[test]
    fn test_remove_field_processor() {
        let processor = RemoveFieldProcessor::new("secret");
        let event = Event::new(json!({
            "message": "test",
            "secret": "password"
        }));
        
        let processed = processor.process(event).unwrap();
        assert!(processed.get("secret").is_none());
        assert_eq!(processed.get("message").unwrap(), "test");
    }
    
    #[test]
    fn test_rename_field_processor() {
        let processor = RenameFieldProcessor::new("old", "new");
        let event = Event::new(json!({
            "old": "value",
            "other": "data"
        }));
        
        let processed = processor.process(event).unwrap();
        assert!(processed.get("old").is_none());
        assert_eq!(processed.get("new").unwrap(), "value");
        assert_eq!(processed.get("other").unwrap(), "data");
    }
    
    #[test]
    fn test_filter_processor() {
        let processor = FilterProcessor::new("level_filter", |event| {
            event.get("level").map_or(false, |v| v != "debug")
        });
        
        // Event that should pass
        let event1 = Event::new(json!({"level": "info"}));
        let result1 = processor.process(event1);
        assert!(result1.is_ok());
        
        // Event that should be filtered
        let event2 = Event::new(json!({"level": "debug"}));
        let result2 = processor.process(event2);
        assert!(result2.is_err());
    }
    
    #[test]
    fn test_simple_processor() {
        let processor = SimpleProcessor::new("test", "Test processor");
        let event = Event::new(json!({"data": "test"}));
        
        let processed = processor.process(event).unwrap();
        assert_eq!(processed.get("data").unwrap(), "test");
        assert_eq!(processor.name(), "test");
        assert_eq!(processor.description(), "Test processor");
    }
}