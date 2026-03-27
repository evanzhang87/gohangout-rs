//! Core traits for event processing

use crate::event::Event;
use std::error::Error;

/// Trait for event operations
pub trait EventTrait {
    /// Get event ID
    fn id(&self) -> uuid::Uuid;
    
    /// Get event timestamp
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;
    
    /// Get event data
    fn data(&self) -> &serde_json::Value;
    
    /// Get mutable event data
    fn data_mut(&mut self) -> &mut serde_json::Value;
    
    /// Get a field from event data
    fn get(&self, field: &str) -> Option<&serde_json::Value>;
    
    /// Set a field in event data
    fn set(&mut self, field: &str, value: serde_json::Value);
    
    /// Remove a field from event data
    fn remove(&mut self, field: &str) -> Option<serde_json::Value>;
    
    /// Check if event contains a field
    fn contains(&self, field: &str) -> bool;
    
    /// Convert event to JSON
    fn to_json(&self) -> Result<String, Box<dyn Error>>;
    
    /// Create event from JSON
    fn from_json(json_str: &str) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

/// Trait for event processors
pub trait ProcessorTrait {
    /// Process a single event
    fn process(&self, event: Event) -> Result<Event, Box<dyn Error>>;
    
    /// Get processor name for logging/debugging
    fn name(&self) -> &str;
}

/// Trait for event processing pipelines
pub trait PipelineTrait {
    /// Process a single event through the pipeline
    fn process_event(&self, event: Event) -> Result<Event, Box<dyn Error>>;
    
    /// Process multiple events through the pipeline
    fn process_batch(&self, events: Vec<Event>) -> Result<Vec<Event>, Box<dyn Error>>;
    
    /// Add a processor to the pipeline
    fn add_processor(&mut self, processor: Box<dyn ProcessorTrait>);
    
    /// Get the number of processors in the pipeline
    fn processor_count(&self) -> usize;
    
    /// Clear all processors from the pipeline
    fn clear_processors(&mut self);
}

// Implement EventTrait for Event
impl EventTrait for Event {
    fn id(&self) -> uuid::Uuid {
        self.id()
    }
    
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp()
    }
    
    fn data(&self) -> &serde_json::Value {
        self.data()
    }
    
    fn data_mut(&mut self) -> &mut serde_json::Value {
        self.data_mut()
    }
    
    fn get(&self, field: &str) -> Option<&serde_json::Value> {
        self.get(field)
    }
    
    fn set(&mut self, field: &str, value: serde_json::Value) {
        self.set(field, value);
    }
    
    fn remove(&mut self, field: &str) -> Option<serde_json::Value> {
        self.remove(field)
    }
    
    fn contains(&self, field: &str) -> bool {
        self.contains(field)
    }
    
    fn to_json(&self) -> Result<String, Box<dyn Error>> {
        self.to_json().map_err(|e| Box::new(e) as Box<dyn Error>)
    }
    
    fn from_json(json_str: &str) -> Result<Self, Box<dyn Error>> {
        Event::from_json(json_str).map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}

/// Type alias for processor function
pub type ProcessorFn = Box<dyn Fn(Event) -> Result<Event, Box<dyn Error>> + Send + Sync>;

/// Simple processor that wraps a function
pub struct FunctionProcessor {
    name: String,
    func: ProcessorFn,
}

impl FunctionProcessor {
    /// Create a new function processor
    pub fn new<F>(name: &str, func: F) -> Self
    where
        F: Fn(Event) -> Result<Event, Box<dyn Error>> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            func: Box::new(func),
        }
    }
}

impl ProcessorTrait for FunctionProcessor {
    fn process(&self, event: Event) -> Result<Event, Box<dyn Error>> {
        (self.func)(event)
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
    fn test_event_trait_impl() {
        let mut event = Event::new(json!({"test": "value"}));
        
        assert!(event.contains("test"));
        assert_eq!(event.get("test").unwrap(), "value");
        
        event.set("new", json!("field"));
        assert_eq!(event.get("new").unwrap(), "field");
        
        let removed = event.remove("test");
        assert_eq!(removed.unwrap(), "value");
        assert!(!event.contains("test"));
    }
    
    #[test]
    fn test_function_processor() {
        let processor = FunctionProcessor::new("test", |mut event| {
            event.set("processed", json!(true));
            Ok(event)
        });
        
        let event = Event::new(json!({}));
        let processed = processor.process(event).unwrap();
        
        assert_eq!(processed.get("processed").unwrap(), true);
        assert_eq!(processor.name(), "test");
    }
}