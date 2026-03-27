//! Decoder implementations for input plugins

use crate::event::Event;
use serde_json::Value;
use std::collections::HashMap;

/// Decoder error type
#[derive(Debug, thiserror::Error)]
pub enum DecoderError {
    /// JSON decoding error
    #[error("JSON decode error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Invalid input format
    #[error("Invalid input format: {0}")]
    InvalidFormat(String),
    
    /// Unsupported codec type
    #[error("Unsupported codec type: {0}")]
    UnsupportedCodec(String),
}

/// Trait for decoding raw data into events
pub trait Decoder: Send + Sync + std::fmt::Debug {
    /// Decode raw data into an event
    fn decode(&self, data: &str) -> Result<Event, DecoderError>;
    
    /// Get decoder name
    fn name(&self) -> &str;
}

/// JSON decoder for JSON-formatted input
#[derive(Debug, Clone)]
pub struct JsonDecoder;

impl JsonDecoder {
    /// Create a new JSON decoder
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for JsonDecoder {
    fn decode(&self, data: &str) -> Result<Event, DecoderError> {
        // Parse JSON
        let value: Value = serde_json::from_str(data)
            .map_err(DecoderError::JsonError)?;
        
        // Create event from JSON value
        Ok(Event::new(value))
    }
    
    fn name(&self) -> &str {
        "json"
    }
}

/// Plain text decoder for plain text input
#[derive(Debug, Clone)]
pub struct PlainDecoder;

impl PlainDecoder {
    /// Create a new plain text decoder
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlainDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for PlainDecoder {
    fn decode(&self, data: &str) -> Result<Event, DecoderError> {
        // Create a simple event with the text as a message field
        let mut event_data = HashMap::new();
        event_data.insert("message".to_string(), Value::String(data.to_string()));
        
        Ok(Event::new(Value::Object(
            event_data.into_iter().collect()
        )))
    }
    
    fn name(&self) -> &str {
        "plain"
    }
}

/// Line decoder for line-oriented text input
#[derive(Debug, Clone)]
pub struct LineDecoder;

impl LineDecoder {
    /// Create a new line decoder
    pub fn new() -> Self {
        Self
    }
}

impl Default for LineDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for LineDecoder {
    fn decode(&self, data: &str) -> Result<Event, DecoderError> {
        // Similar to plain decoder, but might add line-specific metadata
        let mut event_data = HashMap::new();
        event_data.insert("message".to_string(), Value::String(data.to_string()));
        event_data.insert("line".to_string(), Value::String(data.to_string()));
        
        Ok(Event::new(Value::Object(
            event_data.into_iter().collect()
        )))
    }
    
    fn name(&self) -> &str {
        "line"
    }
}

/// Create a decoder based on codec type
pub fn create_decoder(codec_type: &str) -> Result<Box<dyn Decoder>, DecoderError> {
    match codec_type.to_lowercase().as_str() {
        "json" => Ok(Box::new(JsonDecoder::new())),
        "plain" => Ok(Box::new(PlainDecoder::new())),
        "line" => Ok(Box::new(LineDecoder::new())),
        _ => Err(DecoderError::UnsupportedCodec(codec_type.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_json_decoder() {
        let decoder = JsonDecoder::new();
        
        // Valid JSON
        let json = r#"{"test": "value", "number": 42}"#;
        let event = decoder.decode(json).unwrap();
        
        assert_eq!(event.get("test").unwrap(), "value");
        assert_eq!(event.get("number").unwrap(), 42);
        assert_eq!(decoder.name(), "json");
        
        // Invalid JSON
        let invalid = r#"{"invalid: json}"#;
        let result = decoder.decode(invalid);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_plain_decoder() {
        let decoder = PlainDecoder::new();
        
        let text = "Hello, world!";
        let event = decoder.decode(text).unwrap();
        
        assert_eq!(event.get("message").unwrap(), "Hello, world!");
        assert_eq!(decoder.name(), "plain");
    }
    
    #[test]
    fn test_line_decoder() {
        let decoder = LineDecoder::new();
        
        let line = "2024-01-01 INFO: Test message";
        let event = decoder.decode(line).unwrap();
        
        assert_eq!(event.get("message").unwrap(), "2024-01-01 INFO: Test message");
        assert_eq!(event.get("line").unwrap(), "2024-01-01 INFO: Test message");
        assert_eq!(decoder.name(), "line");
    }
    
    #[test]
    fn test_create_decoder() {
        // Test valid codecs
        assert!(create_decoder("json").is_ok());
        assert!(create_decoder("plain").is_ok());
        assert!(create_decoder("line").is_ok());
        
        // Test case insensitive
        assert!(create_decoder("JSON").is_ok());
        assert!(create_decoder("Plain").is_ok());
        assert!(create_decoder("LINE").is_ok());
        
        // Test invalid codec
        let result = create_decoder("invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DecoderError::UnsupportedCodec(_)));
    }
}