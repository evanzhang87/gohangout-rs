//! Configuration module for Gohangout-rs
//!
//! This module handles loading, parsing, and validating configuration files.

mod config;
mod loader;
mod validation;

pub use config::{AppConfig, InputConfig, FilterConfig, OutputConfig};
pub use loader::{ConfigLoader, ConfigError};
pub use validation::ConfigValidator;

/// Default configuration values
pub mod defaults {
    /// Default number of worker threads
    pub const DEFAULT_WORKERS: usize = 4;
    
    /// Default batch size for processing
    pub const DEFAULT_BATCH_SIZE: usize = 1000;
    
    /// Default buffer size for channels
    pub const DEFAULT_BUFFER_SIZE: usize = 10000;
}