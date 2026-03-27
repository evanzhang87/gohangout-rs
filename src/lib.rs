//! Gohangout-rs - A Rust implementation of GoHangout
//!
//! This library provides ETL processing capabilities with multiple input sources,
//! rich filtering options, and various output destinations.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod config;
pub mod event;
pub mod watcher;

/// Re-exports commonly used types
pub mod prelude {
    pub use crate::config::{AppConfig, InputConfig, FilterConfig, OutputConfig};
    pub use crate::event::{Event, EventTrait, PipelineTrait, ProcessorTrait};
    pub use crate::event::{SimplePipeline, PipelineError};
    pub use crate::watcher::ConfigWatcher;
}