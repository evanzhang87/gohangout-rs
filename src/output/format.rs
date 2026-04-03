//! Output formatting utilities

use crate::event::Event;
use serde_json::Value;
use std::collections::HashMap;

/// Output format type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    /// JSON format (compact)
    Json,
    /// JSON format with pretty printing
    Pretty,
    /// Plain text format
    Plain,
}

impl OutputFormat {
    /// Parse format from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "pretty" => Some(Self::Pretty),
            "plain" => Some(Self::Plain),
            _ => None,
        }
    }
    
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Pretty => "pretty",
            Self::Plain => "plain",
        }
    }
}

/// Formatting error
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Formatter trait
pub trait Formatter: Send + Sync {
    /// Format an event
    fn format(&self, event: &Event) -> Result<String, FormatError>;
    
    /// Format an event, ensuring @timestamp exists
    fn format_with_timestamp(&self, event: &Event) -> Result<String, FormatError> {
        let mut event_clone = event.clone();
        event_clone.ensure_timestamp();
        self.format(&event_clone)
    }
    
    /// Format multiple events
    fn format_batch(&self, events: &[Event]) -> Result<String, FormatError> {
        let mut results = Vec::new();
        for event in events {
            results.push(self.format(event)?);
        }
        Ok(results.join("\n"))
    }
    
    /// Format multiple events, ensuring @timestamp exists for each
    fn format_batch_with_timestamp(&self, events: &[Event]) -> Result<String, FormatError> {
        let mut results = Vec::new();
        for event in events {
            results.push(self.format_with_timestamp(event)?);
        }
        Ok(results.join("\n"))
    }
}

/// JSON formatter
pub struct JsonFormatter {
    pretty: bool,
}

impl JsonFormatter {
    /// Create a new JSON formatter
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl Formatter for JsonFormatter {
    fn format(&self, event: &Event) -> Result<String, FormatError> {
        let data = event.data();
        
        if self.pretty {
            serde_json::to_string_pretty(data)
                .map_err(FormatError::JsonError)
        } else {
            serde_json::to_string(data)
                .map_err(FormatError::JsonError)
        }
    }
}

/// Plain text formatter
pub struct PlainFormatter {
    include_timestamp: bool,
    include_fields: Vec<String>,
}

impl PlainFormatter {
    /// Create a new plain formatter
    pub fn new(include_timestamp: bool, include_fields: Vec<String>) -> Self {
        Self {
            include_timestamp,
            include_fields,
        }
    }
    
    /// Extract field value as string
    fn extract_field(&self, data: &Value, field: &str) -> String {
        match data.get(field) {
            Some(value) => match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => "null".to_string(),
                Value::Array(_) | Value::Object(_) => {
                    // For complex values, use JSON representation
                    serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
                }
            },
            None => "-".to_string(),
        }
    }
}

impl Formatter for PlainFormatter {
    fn format(&self, event: &Event) -> Result<String, FormatError> {
        let data = event.data();
        let mut parts = Vec::new();
        
        // Add timestamp if requested
        if self.include_timestamp {
            if let Some(timestamp) = data.get("timestamp") {
                if let Value::String(ts) = timestamp {
                    parts.push(ts.clone());
                }
            }
        }
        
        // Add requested fields
        for field in &self.include_fields {
            let value = self.extract_field(data, field);
            parts.push(value);
        }
        
        // If no specific fields requested, include all simple fields
        if self.include_fields.is_empty() {
            if let Value::Object(map) = data {
                for (key, value) in map {
                    match value {
                        Value::String(s) => parts.push(format!("{}={}", key, s)),
                        Value::Number(n) => parts.push(format!("{}={}", key, n)),
                        Value::Bool(b) => parts.push(format!("{}={}", key, b)),
                        _ => {} // Skip complex types
                    }
                }
            }
        }
        
        Ok(parts.join(" "))
    }
}

/// Pretty formatter (human-readable)
pub struct PrettyFormatter {
    color: bool,
}

impl PrettyFormatter {
    /// Create a new pretty formatter
    pub fn new(color: bool) -> Self {
        Self { color }
    }
    
    /// Colorize output if enabled
    fn colorize(&self, text: &str, color_code: &str) -> String {
        if self.color {
            format!("\x1b[{}m{}\x1b[0m", color_code, text)
        } else {
            text.to_string()
        }
    }
}

impl Formatter for PrettyFormatter {
    fn format(&self, event: &Event) -> Result<String, FormatError> {
        let data = event.data();
        let mut output = String::new();
        
        // Header
        output.push_str(&self.colorize("=== Event ===", "1;36"));
        output.push('\n');
        
        // Timestamp
        if let Some(timestamp) = data.get("timestamp") {
            if let Value::String(ts) = timestamp {
                output.push_str(&format!("{}: {}\n", 
                    self.colorize("Timestamp", "1;33"), 
                    self.colorize(ts.as_str(), "1;37")));
            }
        }
        
        // Level (if present)
        if let Some(level) = data.get("level") {
            if let Value::String(lvl) = level {
                let color = match lvl.as_str() {
                    "ERROR" => "1;31", // Red
                    "WARN" => "1;33",  // Yellow
                    "INFO" => "1;32",  // Green
                    "DEBUG" => "1;34", // Blue
                    _ => "1;37",       // White
                };
                output.push_str(&format!("{}: {}\n",
                    self.colorize("Level", "1;33"),
                    self.colorize(lvl.as_str(), color)));
            }
        }
        
        // Message (if present)
        if let Some(message) = data.get("message") {
            if let Value::String(msg) = message {
                output.push_str(&format!("{}: {}\n",
                    self.colorize("Message", "1;33"),
                    self.colorize(msg.as_str(), "1;37")));
            }
        }
        
        // Other fields
        if let Value::Object(map) = data {
            for (key, value) in map {
                // Skip already handled fields
                if matches!(key.as_str(), "timestamp" | "level" | "message") {
                    continue;
                }
                
                let value_str = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    Value::Array(arr) => {
                        let items: Vec<String> = arr.iter()
                            .map(|v| v.to_string())
                            .collect();
                        format!("[{}]", items.join(", "))
                    }
                    Value::Object(obj) => {
                        let pairs: Vec<String> = obj.iter()
                            .map(|(k, v)| format!("{}: {}", k, v))
                            .collect();
                        format!("{{{}}}", pairs.join(", "))
                    }
                };
                
                output.push_str(&format!("{}: {}\n",
                    self.colorize(&key, "1;33"),
                    self.colorize(&value_str, "1;37")));
            }
        }
        
        output.push_str(&self.colorize("=============", "1;36"));
        
        Ok(output)
    }
}

/// Create formatter based on format type and configuration
pub fn create_formatter(
    format: OutputFormat,
    config: &HashMap<String, serde_json::Value>,
) -> Result<Box<dyn Formatter>, FormatError> {
    match format {
        OutputFormat::Json => {
            let pretty = config.get("pretty")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Box::new(JsonFormatter::new(pretty)))
        }
        OutputFormat::Pretty => {
            let color = config.get("color")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            Ok(Box::new(PrettyFormatter::new(color)))
        }
        OutputFormat::Plain => {
            let include_timestamp = config.get("timestamp")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            
            let include_fields = config.get("fields")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_else(Vec::new);
            
            Ok(Box::new(PlainFormatter::new(include_timestamp, include_fields)))
        }
    }
}