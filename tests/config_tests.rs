//! Configuration module tests

use gohangout_rs::config::*;
use serde_yaml;
use tempfile::NamedTempFile;
use std::fs;

#[test]
fn test_config_deserialization() {
    let yaml = r#"
workers: 4
batch_size: 1000
inputs:
  - type: kafka
    config:
      brokers: "localhost:9092"
      topic: "logs"
  - type: stdin
    config:
      codec: "json"
filters:
  - type: add
    config:
      field: "host"
      value: "localhost"
outputs:
  - type: elasticsearch
    config:
      hosts: ["http://localhost:9200"]
      index: "logs"
"#;

    let config: AppConfig = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
    
    assert_eq!(config.workers, 4);
    assert_eq!(config.batch_size, 1000);
    assert_eq!(config.inputs.len(), 2);
    assert_eq!(config.filters.len(), 1);
    assert_eq!(config.outputs.len(), 1);
    
    // Check first input
    let kafka_input = &config.inputs[0];
    assert_eq!(kafka_input.r#type, "kafka");
    assert_eq!(
        kafka_input.config.get("brokers").unwrap().as_str().unwrap(),
        "localhost:9092"
    );
}

#[test]
fn test_config_loader_from_file() {
    let yaml = r#"
workers: 2
batch_size: 500
inputs: []
filters: []
outputs: []
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(temp_file.path(), yaml).expect("Failed to write temp file");
    
    let loader = ConfigLoader::new();
    let config = loader
        .load_from_file(temp_file.path())
        .expect("Failed to load config from file");
    
    assert_eq!(config.workers, 2);
    assert_eq!(config.batch_size, 500);
}

#[test]
fn test_config_validation() {
    let config = AppConfig {
        workers: 0, // Invalid: workers must be > 0
        batch_size: 1000,
        inputs: vec![],
        filters: vec![],
        outputs: vec![],
    };
    
    let validator = ConfigValidator::new();
    let result = validator.validate(&config);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("workers"));
}

#[test]
fn test_config_default_values() {
    let yaml = r#"
inputs: []
filters: []
outputs: []
"#;

    let config: AppConfig = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
    
    // Should use defaults when not specified
    assert_eq!(config.workers, defaults::DEFAULT_WORKERS);
    assert_eq!(config.batch_size, defaults::DEFAULT_BATCH_SIZE);
}

#[test]
fn test_invalid_yaml() {
    let invalid_yaml = r#"
workers: "not_a_number"  # Should be number
inputs: []
filters: []
outputs: []
"#;

    let result: Result<AppConfig, _> = serde_yaml::from_str(invalid_yaml);
    assert!(result.is_err());
}

#[test]
fn test_missing_required_fields() {
    let incomplete_yaml = r#"
# Missing inputs, filters, outputs
workers: 4
batch_size: 1000
"#;

    let result: Result<AppConfig, _> = serde_yaml::from_str(incomplete_yaml);
    // This depends on whether fields are optional or required
    // For now, we expect it to fail since inputs/filters/outputs are required
    assert!(result.is_err());
}