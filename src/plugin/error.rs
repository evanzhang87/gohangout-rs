//! Plugin error types

use thiserror::Error;

/// Plugin-related errors
#[derive(Debug, Error)]
pub enum PluginError {
    /// Plugin not found in registry
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    /// Plugin initialization failed
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    
    /// Plugin configuration error
    #[error("Plugin configuration error: {0}")]
    ConfigurationError(String),
    
    /// Plugin execution error
    #[error("Plugin execution error: {0}")]
    ExecutionError(String),
    
    /// Plugin registration error (duplicate, etc.)
    #[error("Plugin registration error: {0}")]
    RegistrationError(String),
    
    /// Plugin type mismatch
    #[error("Plugin type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: String,
        actual: String,
    },
    
    /// Invalid plugin name
    #[error("Invalid plugin name: {0}")]
    InvalidName(String),
    
    /// Missing required configuration
    #[error("Missing required configuration: {0}")]
    MissingConfiguration(String),
    
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Other errors
    #[error("Plugin error: {0}")]
    Other(String),
}

impl PluginError {
    /// Create a not found error
    pub fn not_found(name: &str) -> Self {
        Self::NotFound(name.to_string())
    }
    
    /// Create an initialization failed error
    pub fn initialization_failed(reason: &str) -> Self {
        Self::InitializationFailed(reason.to_string())
    }
    
    /// Create a configuration error
    pub fn configuration_error(reason: &str) -> Self {
        Self::ConfigurationError(reason.to_string())
    }
    
    /// Create an execution error
    pub fn execution_error(reason: &str) -> Self {
        Self::ExecutionError(reason.to_string())
    }
    
    /// Create a registration error
    pub fn registration_error(reason: &str) -> Self {
        Self::RegistrationError(reason.to_string())
    }
    
    /// Create a type mismatch error
    pub fn type_mismatch(expected: &str, actual: &str) -> Self {
        Self::TypeMismatch {
            expected: expected.to_string(),
            actual: actual.to_string(),
        }
    }
    
    /// Create an invalid name error
    pub fn invalid_name(name: &str) -> Self {
        Self::InvalidName(name.to_string())
    }
    
    /// Create a missing configuration error
    pub fn missing_configuration(key: &str) -> Self {
        Self::MissingConfiguration(key.to_string())
    }
    
    /// Create an other error
    pub fn other(reason: &str) -> Self {
        Self::Other(reason.to_string())
    }
    
    /// Check if error is a not found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }
    
    /// Check if error is an initialization error
    pub fn is_initialization(&self) -> bool {
        matches!(self, Self::InitializationFailed(_))
    }
    
    /// Check if error is a configuration error
    pub fn is_configuration(&self) -> bool {
        matches!(self, Self::ConfigurationError(_))
    }
    
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        !matches!(self, Self::InitializationFailed(_) | Self::TypeMismatch { .. })
    }
}

/// Result type for plugin operations
pub type PluginResult<T> = std::result::Result<T, PluginError>;