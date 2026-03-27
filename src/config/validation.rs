//! Configuration validation

use crate::config::AppConfig;
use std::collections::HashSet;

/// Configuration validator
#[derive(Debug, Clone)]
pub struct ConfigValidator {
    /// Allowed input types
    allowed_inputs: HashSet<String>,
    
    /// Allowed filter types
    allowed_filters: HashSet<String>,
    
    /// Allowed output types
    allowed_outputs: HashSet<String>,
}

impl Default for ConfigValidator {
    fn default() -> Self {
        let mut allowed_inputs = HashSet::new();
        allowed_inputs.insert("kafka".to_string());
        allowed_inputs.insert("stdin".to_string());
        allowed_inputs.insert("tcp".to_string());
        allowed_inputs.insert("udp".to_string());
        
        let mut allowed_filters = HashSet::new();
        allowed_filters.insert("add".to_string());
        allowed_filters.insert("drop".to_string());
        allowed_filters.insert("convert".to_string());
        allowed_filters.insert("date".to_string());
        allowed_filters.insert("grok".to_string());
        allowed_filters.insert("gsub".to_string());
        
        let mut allowed_outputs = HashSet::new();
        allowed_outputs.insert("elasticsearch".to_string());
        allowed_outputs.insert("clickhouse".to_string());
        allowed_outputs.insert("kafka".to_string());
        allowed_outputs.insert("influxdb".to_string());
        allowed_outputs.insert("stdout".to_string());
        allowed_outputs.insert("tcp".to_string());
        
        Self {
            allowed_inputs,
            allowed_filters,
            allowed_outputs,
        }
    }
}

impl ConfigValidator {
    /// Create a new configuration validator with default allowed types
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a configuration validator with custom allowed types
    pub fn with_allowed_types(
        allowed_inputs: Vec<String>,
        allowed_filters: Vec<String>,
        allowed_outputs: Vec<String>,
    ) -> Self {
        Self {
            allowed_inputs: allowed_inputs.into_iter().collect(),
            allowed_filters: allowed_filters.into_iter().collect(),
            allowed_outputs: allowed_outputs.into_iter().collect(),
        }
    }
    
    /// Validate the entire configuration
    pub fn validate(&self, config: &AppConfig) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Validate basic fields
        if config.workers == 0 {
            errors.push("workers must be greater than 0".to_string());
        }
        
        if config.batch_size == 0 {
            errors.push("batch_size must be greater than 0".to_string());
        }
        
        if config.buffer_size == 0 {
            errors.push("buffer_size must be greater than 0".to_string());
        }
        
        // Validate inputs
        if config.inputs.is_empty() {
            errors.push("at least one input must be configured".to_string());
        }
        
        for (i, input) in config.inputs.iter().enumerate() {
            if !self.allowed_inputs.contains(&input.r#type) {
                errors.push(format!(
                    "input[{}]: unknown input type '{}'. Allowed: {}",
                    i,
                    input.r#type,
                    self.allowed_inputs.iter().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
            
            if input.config.is_empty() {
                errors.push(format!(
                    "input[{}] (type: {}): configuration is empty",
                    i, input.r#type
                ));
            }
        }
        
        // Validate filters
        for (i, filter) in config.filters.iter().enumerate() {
            if !self.allowed_filters.contains(&filter.r#type) {
                errors.push(format!(
                    "filter[{}]: unknown filter type '{}'. Allowed: {}",
                    i,
                    filter.r#type,
                    self.allowed_filters.iter().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
            
            if filter.config.is_empty() {
                errors.push(format!(
                    "filter[{}] (type: {}): configuration is empty",
                    i, filter.r#type
                ));
            }
        }
        
        // Validate outputs
        if config.outputs.is_empty() {
            errors.push("at least one output must be configured".to_string());
        }
        
        for (i, output) in config.outputs.iter().enumerate() {
            if !self.allowed_outputs.contains(&output.r#type) {
                errors.push(format!(
                    "output[{}]: unknown output type '{}'. Allowed: {}",
                    i,
                    output.r#type,
                    self.allowed_outputs.iter().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
            
            if output.config.is_empty() {
                errors.push(format!(
                    "output[{}] (type: {}): configuration is empty",
                    i, output.r#type
                ));
            }
        }
        
        // Check for duplicate plugin configurations
        self.check_duplicates(config, &mut errors);
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Check for duplicate plugin configurations
    fn check_duplicates(&self, config: &AppConfig, errors: &mut Vec<String>) {
        // Check for duplicate input types with identical configs
        let mut seen_inputs = HashSet::new();
        for (i, input) in config.inputs.iter().enumerate() {
            let key = format!("{}:{:?}", input.r#type, input.config);
            if seen_inputs.contains(&key) {
                errors.push(format!(
                    "input[{}]: duplicate configuration for type '{}'",
                    i, input.r#type
                ));
            }
            seen_inputs.insert(key);
        }
        
        // Similar checks for filters and outputs could be added
    }
    
    /// Get allowed input types
    pub fn allowed_inputs(&self) -> &HashSet<String> {
        &self.allowed_inputs
    }
    
    /// Get allowed filter types
    pub fn allowed_filters(&self) -> &HashSet<String> {
        &self.allowed_filters
    }
    
    /// Get allowed output types
    pub fn allowed_outputs(&self) -> &HashSet<String> {
        &self.allowed_outputs
    }
    
    /// Add a new allowed input type
    pub fn add_input_type(&mut self, input_type: String) {
        self.allowed_inputs.insert(input_type);
    }
    
    /// Add a new allowed filter type
    pub fn add_filter_type(&mut self, filter_type: String) {
        self.allowed_filters.insert(filter_type);
    }
    
    /// Add a new allowed output type
    pub fn add_output_type(&mut self, output_type: String) {
        self.allowed_outputs.insert(output_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{InputConfig, OutputConfig, FilterConfig};
    use std::collections::HashMap;
    
    #[test]
    fn test_validate_valid_config() {
        let config = AppConfig {
            workers: 4,
            batch_size: 1000,
            buffer_size: 10000,
            inputs: vec![
                InputConfig {
                    r#type: "stdin".to_string(),
                    config: {
                        let mut map = HashMap::new();
                        map.insert("codec".to_string(), "json".into());
                        map
                    },
                },
            ],
            filters: vec![
                FilterConfig {
                    r#type: "add".to_string(),
                    config: {
                        let mut map = HashMap::new();
                        map.insert("field".to_string(), "host".into());
                        map.insert("value".to_string(), "localhost".into());
                        map
                    },
                    condition: None,
                },
            ],
            outputs: vec![
                OutputConfig {
                    r#type: "stdout".to_string(),
                    config: {
                        let mut map = HashMap::new();
                        map.insert("format".to_string(), "json".into());
                        map
                    },
                },
            ],
        };
        
        let validator = ConfigValidator::new();
        let result = validator.validate(&config);
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_invalid_workers() {
        let config = AppConfig {
            workers: 0, // Invalid
            batch_size: 1000,
            buffer_size: 10000,
            inputs: vec![InputConfig {
                r#type: "stdin".to_string(),
                config: HashMap::new(),
            }],
            filters: vec![],
            outputs: vec![OutputConfig {
                r#type: "stdout".to_string(),
                config: HashMap::new(),
            }],
        };
        
        let validator = ConfigValidator::new();
        let result = validator.validate(&config);
        
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("workers")));
    }
    
    #[test]
    fn test_validate_unknown_input_type() {
        let config = AppConfig {
            workers: 4,
            batch_size: 1000,
            buffer_size: 10000,
            inputs: vec![InputConfig {
                r#type: "unknown_input".to_string(), // Not in allowed list
                config: HashMap::new(),
            }],
            filters: vec![],
            outputs: vec![OutputConfig {
                r#type: "stdout".to_string(),
                config: HashMap::new(),
            }],
        };
        
        let validator = ConfigValidator::new();
        let result = validator.validate(&config);
        
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("unknown_input")));
    }
    
    #[test]
    fn test_validate_empty_config() {
        let config = AppConfig {
            workers: 4,
            batch_size: 1000,
            buffer_size: 10000,
            inputs: vec![], // Empty - should error
            filters: vec![],
            outputs: vec![], // Empty - should error
        };
        
        let validator = ConfigValidator::new();
        let result = validator.validate(&config);
        
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("at least one input")));
        assert!(errors.iter().any(|e| e.contains("at least one output")));
    }
    
    #[test]
    fn test_custom_allowed_types() {
        let validator = ConfigValidator::with_allowed_types(
            vec!["custom_input".to_string()],
            vec!["custom_filter".to_string()],
            vec!["custom_output".to_string()],
        );
        
        let config = AppConfig {
            workers: 4,
            batch_size: 1000,
            buffer_size: 10000,
            inputs: vec![InputConfig {
                r#type: "custom_input".to_string(),
                config: HashMap::new(),
            }],
            filters: vec![FilterConfig {
                r#type: "custom_filter".to_string(),
                config: HashMap::new(),
                condition: None,
            }],
            outputs: vec![OutputConfig {
                r#type: "custom_output".to_string(),
                config: HashMap::new(),
            }],
        };
        
        let result = validator.validate(&config);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_duplicate_inputs() {
        let mut config = HashMap::new();
        config.insert("codec".to_string(), "json".into());
        
        let config = AppConfig {
            workers: 4,
            batch_size: 1000,
            buffer_size: 10000,
            inputs: vec![
                InputConfig {
                    r#type: "stdin".to_string(),
                    config: config.clone(),
                },
                InputConfig {
                    r#type: "stdin".to_string(),
                    config: config.clone(), // Same config - duplicate
                },
            ],
            filters: vec![],
            outputs: vec![OutputConfig {
                r#type: "stdout".to_string(),
                config: HashMap::new(),
            }],
        };
        
        let validator = ConfigValidator::new();
        let result = validator.validate(&config);
        
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("duplicate")));
    }
}