//! Pipeline implementation for event processing

use crate::event::{Event, ProcessorTrait};
use std::error::Error;
use thiserror::Error;

/// Pipeline errors
#[derive(Debug, Error)]
pub enum PipelineError {
    /// Processor error
    #[error("Processor error: {0}")]
    ProcessorError(String),
    
    /// Batch processing error
    #[error("Batch processing error at index {index}: {error}")]
    BatchError {
        index: usize,
        error: String,
    },
    
    /// Empty pipeline error
    #[error("Pipeline has no processors")]
    EmptyPipeline,
}

/// Simple pipeline implementation
pub struct SimplePipeline {
    processors: Vec<Box<dyn ProcessorTrait>>,
    name: String,
}

impl SimplePipeline {
    /// Create a new pipeline
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
            name: "default".to_string(),
        }
    }
    
    /// Create a new pipeline with a name
    pub fn with_name(name: &str) -> Self {
        Self {
            processors: Vec::new(),
            name: name.to_string(),
        }
    }
    
    /// Get pipeline name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Set pipeline name
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
    
    /// Process a single event through all processors
    pub fn process(&self, event: Event) -> Result<Event, PipelineError> {
        if self.processors.is_empty() {
            return Ok(event);
        }
        
        let mut current_event = event;
        
        for (i, processor) in self.processors.iter().enumerate() {
            match processor.process(current_event) {
                Ok(processed) => {
                    current_event = processed;
                }
                Err(e) => {
                    return Err(PipelineError::ProcessorError(format!(
                        "Processor '{}' (index {}) failed: {}",
                        processor.name(),
                        i,
                        e
                    )));
                }
            }
        }
        
        Ok(current_event)
    }
    
    /// Process a batch of events
    pub fn process_batch(&self, events: Vec<Event>) -> Result<Vec<Event>, PipelineError> {
        let mut results = Vec::with_capacity(events.len());
        
        for (i, event) in events.into_iter().enumerate() {
            match self.process(event) {
                Ok(processed) => results.push(processed),
                Err(e) => {
                    return Err(PipelineError::BatchError {
                        index: i,
                        error: e.to_string(),
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// Add a processor to the pipeline
    pub fn add_processor(&mut self, processor: Box<dyn ProcessorTrait>) {
        self.processors.push(processor);
    }
    
    /// Add a function as a processor
    pub fn add_function<F>(&mut self, name: &str, func: F)
    where
        F: Fn(Event) -> Result<Event, Box<dyn Error>> + Send + Sync + 'static,
    {
        use crate::event::traits::FunctionProcessor;
        
        let processor = FunctionProcessor::new(name, func);
        self.add_processor(Box::new(processor));
    }
    
    /// Get the number of processors
    pub fn processor_count(&self) -> usize {
        self.processors.len()
    }
    
    /// Clear all processors
    pub fn clear_processors(&mut self) {
        self.processors.clear();
    }
    
    /// Get processor at index
    pub fn get_processor(&self, index: usize) -> Option<&dyn ProcessorTrait> {
        self.processors.get(index).map(|p| p.as_ref())
    }
    
    /// Remove processor at index
    pub fn remove_processor(&mut self, index: usize) -> Option<Box<dyn ProcessorTrait>> {
        if index < self.processors.len() {
            Some(self.processors.remove(index))
        } else {
            None
        }
    }
}

impl Default for SimplePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::event::traits::PipelineTrait for SimplePipeline {
    fn process_event(&self, event: Event) -> Result<Event, Box<dyn Error>> {
        self.process(event).map_err(|e| Box::new(e) as Box<dyn Error>)
    }
    
    fn process_batch(&self, events: Vec<Event>) -> Result<Vec<Event>, Box<dyn Error>> {
        self.process_batch(events).map_err(|e| Box::new(e) as Box<dyn Error>)
    }
    
    fn add_processor(&mut self, processor: Box<dyn ProcessorTrait>) {
        self.add_processor(processor);
    }
    
    fn processor_count(&self) -> usize {
        self.processor_count()
    }
    
    fn clear_processors(&mut self) {
        self.clear_processors();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::traits::FunctionProcessor;
    use serde_json::json;
    
    #[test]
    fn test_pipeline_basic() {
        let mut pipeline = SimplePipeline::new();
        
        // Add a processor that adds a field
        pipeline.add_function("add_field", |mut event| {
            event.set("processed", json!(true));
            Ok(event)
        });
        
        // Add a processor that modifies a field
        pipeline.add_function("modify", |mut event| {
            if let Some(value) = event.get("count") {
                if let Some(num) = value.as_i64() {
                    event.set("count", json!(num + 1));
                }
            }
            Ok(event)
        });
        
        let event = Event::new(json!({"count": 0}));
        let processed = pipeline.process(event).unwrap();
        
        assert_eq!(processed.get("processed").unwrap(), true);
        assert_eq!(processed.get("count").unwrap(), 1);
        assert_eq!(pipeline.processor_count(), 2);
    }
    
    #[test]
    fn test_pipeline_batch() {
        let mut pipeline = SimplePipeline::new();
        
        pipeline.add_function("increment", |mut event| {
            if let Some(value) = event.get("id") {
                if let Some(num) = value.as_i64() {
                    event.set("id", json!(num + 100));
                }
            }
            Ok(event)
        });
        
        let events = vec![
            Event::new(json!({"id": 1})),
            Event::new(json!({"id": 2})),
            Event::new(json!({"id": 3})),
        ];
        
        let processed = pipeline.process_batch(events).unwrap();
        
        assert_eq!(processed.len(), 3);
        assert_eq!(processed[0].get("id").unwrap(), 101);
        assert_eq!(processed[1].get("id").unwrap(), 102);
        assert_eq!(processed[2].get("id").unwrap(), 103);
    }
    
    #[test]
    fn test_pipeline_error() {
        let mut pipeline = SimplePipeline::new();
        
        pipeline.add_function("error", |_| {
            Err("Test error".into())
        });
        
        let event = Event::new(json!({}));
        let result = pipeline.process(event);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Test error"));
    }
    
    #[test]
    fn test_pipeline_processor_management() {
        let mut pipeline = SimplePipeline::new();
        
        assert_eq!(pipeline.processor_count(), 0);
        
        // Add processors
        pipeline.add_function("p1", |e| Ok(e));
        pipeline.add_function("p2", |e| Ok(e));
        
        assert_eq!(pipeline.processor_count(), 2);
        
        // Remove processor
        let removed = pipeline.remove_processor(0);
        assert!(removed.is_some());
        assert_eq!(pipeline.processor_count(), 1);
        
        // Clear processors
        pipeline.clear_processors();
        assert_eq!(pipeline.processor_count(), 0);
    }
    
    #[test]
    fn test_empty_pipeline() {
        let pipeline = SimplePipeline::new();
        let event = Event::new(json!({"test": "value"}));
        
        // Empty pipeline should pass through unchanged
        let processed = pipeline.process(event).unwrap();
        assert_eq!(processed.get("test").unwrap(), "value");
    }
}