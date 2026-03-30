//! RandomInput plugin implementation

use crate::event::Event;
use crate::prelude::{PluginError, PluginResult, Input, InputStats, Plugin, PluginType};
use crate::plugin::PluginConfig;
use crate::plugin::traits::PluginStatus;
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use uuid::Uuid;

/// Random data generation mode
#[derive(Debug, Clone, PartialEq)]
pub enum RandomMode {
    /// Simple mode: basic fields (timestamp, message, level)
    Simple,
    /// Complex mode: nested JSON structure
    Complex,
    /// Custom mode: user-defined template
    Custom,
}

impl RandomMode {
    /// Parse mode from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "simple" => Some(Self::Simple),
            "complex" => Some(Self::Complex),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
    
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Complex => "complex",
            Self::Custom => "custom",
        }
    }
}

/// Field type for custom mode
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    DateTime,
    Uuid,
    Email,
    IpAddress,
    Enum(Vec<String>),
}

impl FieldType {
    /// Parse field type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "string" => Some(Self::String),
            "number" => Some(Self::Number),
            "boolean" => Some(Self::Boolean),
            "datetime" => Some(Self::DateTime),
            "uuid" => Some(Self::Uuid),
            "email" => Some(Self::Email),
            "ip" | "ipaddress" => Some(Self::IpAddress),
            _ => {
                if s.starts_with("enum:") {
                    let values: Vec<String> = s[5..]
                        .split(',')
                        .map(|v| v.trim().to_string())
                        .collect();
                    if values.is_empty() {
                        None
                    } else {
                        Some(Self::Enum(values))
                    }
                } else {
                    None
                }
            }
        }
    }
    
    /// Generate random value for this field type
    pub fn generate_random(&self, rng: &mut StdRng) -> Value {
        match self {
            Self::String => {
                let length = rng.gen_range(5..=50);
                let s: String = rng
                    .sample_iter(&Alphanumeric)
                    .take(length)
                    .map(char::from)
                    .collect();
                Value::String(s)
            }
            Self::Number => {
                if rng.gen_bool(0.5) {
                    Value::Number(rng.gen_range(0..=1000).into())
                } else {
                    Value::Number(serde_json::Number::from_f64(rng.gen_range(0.0..=1000.0)).unwrap_or(serde_json::Number::from(0)))
                }
            }
            Self::Boolean => Value::Bool(rng.r#gen()),
            Self::DateTime => {
                let now = Utc::now();
                let offset = rng.gen_range(-86400..86400); // +/- 1 day
                let dt = now + chrono::Duration::seconds(offset);
                Value::String(dt.to_rfc3339())
            }
            Self::Uuid => Value::String(Uuid::new_v4().to_string()),
            Self::Email => {
                let username: String = rng
                    .sample_iter(&Alphanumeric)
                    .take(8)
                    .map(char::from)
                    .collect();
                let domains = ["example.com", "test.com", "demo.org", "sample.net"];
                let domain = domains[rng.gen_range(0..domains.len())];
                Value::String(format!("{}@{}", username, domain))
            }
            Self::IpAddress => {
                let octets: Vec<u8> = (0..4).map(|_| rng.gen_range(1..=254)).collect();
                Value::String(format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3]))
            }
            Self::Enum(values) => {
                let idx = rng.gen_range(0..values.len());
                Value::String(values[idx].clone())
            }
        }
    }
}

/// RandomInput plugin for generating random test data
pub struct RandomInput {
    /// Plugin name
    name: String,
    
    /// Plugin configuration
    config: HashMap<String, Value>,
    
    /// Generation mode
    mode: RandomMode,
    
    /// Events per second (0 = as fast as possible)
    rate: u32,
    
    /// Total events to generate (0 = infinite)
    count: u64,
    
    /// Custom field definitions (for custom mode)
    fields: HashMap<String, FieldType>,
    
    /// Random number generator
    rng: StdRng,
    
    /// Statistics
    stats: Arc<Mutex<InputStats>>,
    
    /// Events generated so far
    generated: u64,
    
    /// Last generation time (for rate limiting)
    last_generation: Option<Instant>,
    
    /// Start time
    start_time: Instant,
}

impl RandomInput {
    /// Create a new RandomInput from configuration
    pub fn from_config(config: &PluginConfig) -> PluginResult<Self> {
        // Get plugin name
        let name = config.name.clone();
        
        // Get mode (default: simple)
        let mode_str = config
            .get_config("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("simple");
        let mode = RandomMode::from_str(mode_str)
            .ok_or_else(|| PluginError::ConfigurationError(
                format!("Invalid mode: {}", mode_str)
            ))?;
        
        // Get rate (default: 10 events/second)
        let rate = config
            .get_config("rate")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as u32;
        
        // Get count (default: 0 = infinite)
        let count = config
            .get_config("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        // Parse custom fields if in custom mode
        let mut fields = HashMap::new();
        if mode == RandomMode::Custom {
            if let Some(fields_config) = config.get_config("fields") {
                if let Some(fields_map) = fields_config.as_object() {
                    for (field_name, field_type_value) in fields_map {
                        if let Some(field_type_str) = field_type_value.as_str() {
                            if let Some(field_type) = FieldType::from_str(field_type_str) {
                                fields.insert(field_name.clone(), field_type);
                            } else {
                                return Err(PluginError::ConfigurationError(
                                    format!("Invalid field type for '{}': {}", field_name, field_type_str)
                                ));
                            }
                        } else {
                            return Err(PluginError::ConfigurationError(
                                format!("Field type for '{}' must be a string", field_name)
                            ));
                        }
                    }
                } else {
                    return Err(PluginError::ConfigurationError(
                        "Fields configuration must be an object".to_string()
                    ));
                }
            }
            
            // If no fields specified, use default fields
            if fields.is_empty() {
                fields.insert("user_id".to_string(), FieldType::Number);
                fields.insert("username".to_string(), FieldType::String);
                fields.insert("email".to_string(), FieldType::Email);
                fields.insert("active".to_string(), FieldType::Boolean);
            }
        }
        
        // Create random number generator with seed from config or random
        let seed = config
            .get_config("seed")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| rand::random());
        let rng = StdRng::seed_from_u64(seed);
        
        // Extract config map
        let config_map = config.config.clone();
        
        Ok(Self {
            name,
            config: config_map,
            mode,
            rate,
            count,
            fields,
            rng,
            stats: Arc::new(Mutex::new(InputStats::default())),
            generated: 0,
            last_generation: None,
            start_time: Instant::now(),
        })
    }
    
    /// Generate a simple event
    fn generate_simple(&mut self) -> Event {
        let levels = ["INFO", "WARN", "ERROR", "DEBUG"];
        let messages = [
            "Processing request",
            "User logged in",
            "Database query executed",
            "File uploaded",
            "Cache updated",
            "Network request sent",
            "Task completed",
            "Error occurred",
            "Configuration loaded",
            "Service started",
        ];
        
        let level = levels[self.rng.gen_range(0..levels.len())];
        let message = messages[self.rng.gen_range(0..messages.len())];
        let extra_num: u32 = self.rng.gen_range(1..=1000);
        
        let mut data = HashMap::new();
        data.insert("timestamp".to_string(), json!(Utc::now().to_rfc3339()));
        data.insert("level".to_string(), json!(level));
        data.insert("message".to_string(), json!(message));
        data.insert("extra".to_string(), json!(extra_num));
        
        Event::new(Value::Object(data.into_iter().collect()))
    }
    
    /// Generate a complex event
    fn generate_complex(&mut self) -> Event {
        let mut data = HashMap::new();
        
        // Basic fields
        data.insert("id".to_string(), json!(Uuid::new_v4().to_string()));
        data.insert("timestamp".to_string(), json!(Utc::now().to_rfc3339()));
        
        // User object
        let mut user = HashMap::new();
        user.insert("id".to_string(), json!(self.rng.gen_range(1..=10000)));
        
        let first_names = ["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank"];
        let last_names = ["Smith", "Johnson", "Williams", "Brown", "Jones"];
        let first_name = first_names[self.rng.gen_range(0..first_names.len())];
        let last_name = last_names[self.rng.gen_range(0..last_names.len())];
        user.insert("name".to_string(), json!(format!("{} {}", first_name, last_name)));
        
        let domains = ["example.com", "company.org", "test.net"];
        let domain = domains[self.rng.gen_range(0..domains.len())];
        let username: String = self.rng.clone()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        user.insert("email".to_string(), json!(format!("{}@{}", username, domain)));
        user.insert("active".to_string(), json!(self.rng.gen_bool(0.8)));
        
        data.insert("user".to_string(), json!(user));
        
        // Request object
        let mut request = HashMap::new();
        let methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
        let paths = ["/api/users", "/api/products", "/api/orders", "/health", "/metrics"];
        let statuses = [200, 201, 400, 401, 404, 500];
        
        request.insert("method".to_string(), json!(methods[self.rng.gen_range(0..methods.len())]));
        request.insert("path".to_string(), json!(paths[self.rng.gen_range(0..paths.len())]));
        request.insert("status".to_string(), json!(statuses[self.rng.gen_range(0..statuses.len())]));
        request.insert("duration_ms".to_string(), json!(self.rng.gen_range(10..=5000)));
        
        let octets: Vec<u8> = (0..4).map(|_| self.rng.gen_range(1..=254)).collect();
        request.insert("client_ip".to_string(), json!(format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])));
        
        data.insert("request".to_string(), json!(request));
        
        // Metrics
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage".to_string(), json!(self.rng.gen_range(0.0..=100.0)));
        metrics.insert("memory_mb".to_string(), json!(self.rng.gen_range(50..=4096)));
        metrics.insert("queue_length".to_string(), json!(self.rng.gen_range(0..=100)));
        
        data.insert("metrics".to_string(), json!(metrics));
        
        Event::new(Value::Object(data.into_iter().collect()))
    }
    
    /// Generate a custom event based on field definitions
    fn generate_custom(&mut self) -> Event {
        let mut data = HashMap::new();
        
        // Add timestamp and ID by default
        data.insert("timestamp".to_string(), json!(Utc::now().to_rfc3339()));
        data.insert("id".to_string(), json!(Uuid::new_v4().to_string()));
        
        // Generate each custom field
        for (field_name, field_type) in &self.fields {
            data.insert(field_name.clone(), field_type.generate_random(&mut self.rng));
        }
        
        Event::new(Value::Object(data.into_iter().collect()))
    }
    
    /// Apply rate limiting if needed
    fn apply_rate_limit(&mut self) {
        if self.rate > 0 {
            if let Some(last_gen) = self.last_generation {
                let target_interval = Duration::from_secs_f64(1.0 / self.rate as f64);
                let elapsed = last_gen.elapsed();
                
                if elapsed < target_interval {
                    let sleep_duration = target_interval - elapsed;
                    std::thread::sleep(sleep_duration);
                }
            }
            self.last_generation = Some(Instant::now());
        }
    }
}

impl Plugin for RandomInput {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn config(&self) -> &HashMap<String, Value> {
        &self.config
    }
    
    fn plugin_type(&self) -> PluginType {
        PluginType::Input
    }
    
    fn validate_config(&self) -> PluginResult<()> {
        // Validate rate
        if self.rate > 100000 {
            return Err(PluginError::ConfigurationError(
                "Rate cannot exceed 100,000 events per second".to_string()
            ));
        }
        
        // Validate count
        if self.count > 1_000_000_000 {
            return Err(PluginError::ConfigurationError(
                "Count cannot exceed 1,000,000,000".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn initialize(&mut self) -> PluginResult<()> {
        self.start_time = Instant::now();
        self.generated = 0;
        self.last_generation = None;
        Ok(())
    }
    
    fn shutdown(&mut self) -> PluginResult<()> {
        // Update statistics
        let elapsed = self.start_time.elapsed();
        let mut stats = self.stats.lock().unwrap();
        stats.read_time_ms = elapsed.as_millis() as u64;
        stats.events_read = self.generated;
        Ok(())
    }
    
    fn status(&self) -> PluginStatus {
        if self.count > 0 && self.generated >= self.count {
            PluginStatus::Stopped
        } else {
            PluginStatus::Ready
        }
    }
}

impl Input for RandomInput {
    fn read(&mut self) -> PluginResult<Option<Event>> {
        // Check if we've reached the count limit
        if self.count > 0 && self.generated >= self.count {
            return Ok(None);
        }
        
        // Apply rate limiting
        self.apply_rate_limit();
        
        // Generate event based on mode
        let event = match self.mode {
            RandomMode::Simple => self.generate_simple(),
            RandomMode::Complex => self.generate_complex(),
            RandomMode::Custom => self.generate_custom(),
        };
        
        // Update statistics
        self.generated += 1;
        let mut stats = self.stats.lock().unwrap();
        stats.events_read = self.generated;
        // InputStats doesn't have last_event_time field, skipping
        
        Ok(Some(event))
    }
    
    fn is_ready(&self) -> bool {
        self.count == 0 || self.generated < self.count
    }
    
    fn stats(&self) -> InputStats {
        self.stats.lock().unwrap().clone()
    }
}

impl Default for RandomInput {
    fn default() -> Self {
        let config = PluginConfig {
            name: "random".to_string(),
            plugin_type: PluginType::Input,
            config: HashMap::new(),
        };
        
        Self::from_config(&config).unwrap()
    }
}