//! Configuration loading utilities

use crate::config::AppConfig;
use serde_yaml;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during configuration loading
#[derive(Debug, Error)]
pub enum ConfigError {
    /// File not found or cannot be read
    #[error("Failed to read configuration file: {0}")]
    FileReadError(#[source] std::io::Error),
    
    /// YAML parsing error
    #[error("Failed to parse YAML configuration: {0}")]
    ParseError(#[source] serde_yaml::Error),
    
    /// Configuration validation error
    #[error("Configuration validation failed: {0}")]
    ValidationError(String),
    
    /// Environment variable error
    #[error("Failed to read environment variable: {0}")]
    EnvError(#[source] std::env::VarError),
}

/// Configuration loader
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    /// Whether to validate configuration after loading
    validate: bool,
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self { validate: true }
    }
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a configuration loader with validation disabled
    pub fn without_validation() -> Self {
        Self { validate: false }
    }
    
    /// Load configuration from a file
    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> std::result::Result<AppConfig, ConfigError> {
        let path = path.as_ref();
        
        // Read file contents
        let contents = fs::read_to_string(path)
            .map_err(ConfigError::FileReadError)?;
        
        self.load_from_str(&contents)
    }
    
    /// Load configuration from a string
    pub fn load_from_str(&self, contents: &str) -> std::result::Result<AppConfig, ConfigError> {
        // Parse YAML
        let mut config: AppConfig = serde_yaml::from_str(contents)
            .map_err(ConfigError::ParseError)?;
        
        // Apply environment variable overrides
        self.apply_env_overrides(&mut config)?;
        
        // Validate if enabled
        if self.validate {
            config.validate()
                .map_err(ConfigError::ValidationError)?;
        }
        
        Ok(config)
    }
    
    /// Load configuration from multiple files (merge)
    pub fn load_from_files<P: AsRef<Path>>(&self, paths: &[P]) -> std::result::Result<AppConfig, ConfigError> {
        let mut merged_config = AppConfig::new();
        
        for path in paths {
            let config = self.load_from_file(path)?;
            self.merge_configs(&mut merged_config, config);
        }
        
        // Validate merged configuration
        if self.validate {
            merged_config.validate()
                .map_err(ConfigError::ValidationError)?;
        }
        
        Ok(merged_config)
    }
    
    /// Apply environment variable overrides to configuration
    fn apply_env_overrides(&self, config: &mut AppConfig) -> Result<(), ConfigError> {
        // Override workers from environment
        if let Ok(workers_str) = std::env::var("GOHANGOUT_WORKERS") {
            if let Ok(workers) = workers_str.parse::<usize>() {
                config.workers = workers;
            }
        }
        
        // Override batch size from environment
        if let Ok(batch_size_str) = std::env::var("GOHANGOUT_BATCH_SIZE") {
            if let Ok(batch_size) = batch_size_str.parse::<usize>() {
                config.batch_size = batch_size;
            }
        }
        
        Ok(())
    }
    
    /// Merge two configurations (second overrides first)
    fn merge_configs(&self, base: &mut AppConfig, override_config: AppConfig) {
        // For simple fields, override takes precedence if non-default
        if override_config.workers != default_workers() {
            base.workers = override_config.workers;
        }
        
        if override_config.batch_size != default_batch_size() {
            base.batch_size = override_config.batch_size;
        }
        
        if override_config.buffer_size != default_buffer_size() {
            base.buffer_size = override_config.buffer_size;
        }
        
        // For arrays, append (could be more sophisticated merging)
        base.inputs.extend(override_config.inputs);
        base.filters.extend(override_config.filters);
        base.outputs.extend(override_config.outputs);
    }
}

// Helper functions for defaults
fn default_workers() -> usize {
    crate::config::defaults::DEFAULT_WORKERS
}

fn default_batch_size() -> usize {
    crate::config::defaults::DEFAULT_BATCH_SIZE
}

fn default_buffer_size() -> usize {
    crate::config::defaults::DEFAULT_BUFFER_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_valid_config() {
        let yaml = r#"
workers: 2
batch_size: 500
inputs:
  - type: stdin
    config:
      codec: "json"
outputs:
  - type: stdout
    config:
      format: "json"
filters: []
"#;
        
        let loader = ConfigLoader::new();
        let config = loader.load_from_str(yaml).expect("Failed to load config");
        
        assert_eq!(config.workers, 2);
        assert_eq!(config.batch_size, 500);
        assert_eq!(config.inputs.len(), 1);
        assert_eq!(config.outputs.len(), 1);
    }
    
    #[test]
    fn test_load_invalid_config() {
        let yaml = r#"
workers: 0  # Invalid: must be > 0
inputs: []
outputs: []
filters: []
"#;
        
        let loader = ConfigLoader::new();
        let result = loader.load_from_str(yaml);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::ValidationError(_)));
    }
    
    #[test]
    fn test_load_without_validation() {
        let yaml = r#"
workers: 0  # Invalid but validation is disabled
inputs: []
outputs: []
filters: []
"#;
        
        let loader = ConfigLoader::without_validation();
        let config = loader.load_from_str(yaml).expect("Failed to load config");
        
        assert_eq!(config.workers, 0);
    }
    
    #[test]
    fn test_env_overrides() {
        std::env::set_var("GOHANGOUT_WORKERS", "8");
        std::env::set_var("GOHANGOUT_BATCH_SIZE", "2000");
        
        let yaml = r#"
workers: 2
batch_size: 500
inputs:
  - type: stdin
    config: {}
outputs:
  - type: stdout
    config: {}
filters: []
"#;
        
        let loader = ConfigLoader::new();
        let config = loader.load_from_str(yaml).expect("Failed to load config");
        
        // Should be overridden by environment variables
        assert_eq!(config.workers, 8);
        assert_eq!(config.batch_size, 2000);
        
        // Clean up
        std::env::remove_var("GOHANGOUT_WORKERS");
        std::env::remove_var("GOHANGOUT_BATCH_SIZE");
    }
    
    #[test]
    fn test_merge_configs() {
        let base_yaml = r#"
workers: 2
batch_size: 500
inputs:
  - type: stdin
    config: {}
outputs: []
filters: []
"#;
        
        let override_yaml = r#"
workers: 4
batch_size: 1000
inputs:
  - type: kafka
    config:
      brokers: "localhost:9092"
outputs:
  - type: stdout
    config: {}
filters: []
"#;
        
        let temp1 = NamedTempFile::new().unwrap();
        let temp2 = NamedTempFile::new().unwrap();
        
        std::fs::write(temp1.path(), base_yaml).unwrap();
        std::fs::write(temp2.path(), override_yaml).unwrap();
        
        let loader = ConfigLoader::new();
        let config = loader.load_from_files(&[temp1.path(), temp2.path()])
            .expect("Failed to load merged config");
        
        // Should use values from second file
        assert_eq!(config.workers, 4);
        assert_eq!(config.batch_size, 1000);
        
        // Should have inputs from both files
        assert_eq!(config.inputs.len(), 2);
        assert_eq!(config.inputs[0].r#type, "stdin");
        assert_eq!(config.inputs[1].r#type, "kafka");
        
        // Should have outputs from second file
        assert_eq!(config.outputs.len(), 1);
        assert_eq!(config.outputs[0].r#type, "stdout");
    }
}