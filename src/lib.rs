//! Gohangout-rs - A Rust implementation of GoHangout
//!
//! This library provides ETL processing capabilities with multiple input sources,
//! rich filtering options, and various output destinations.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![allow(dead_code)]

pub mod config;
pub mod event;
pub mod input;
pub mod output;
pub mod plugin;
pub mod watcher;

/// Re-exports commonly used types
pub mod prelude {
    pub use crate::config::{AppConfig, InputConfig, FilterConfig, OutputConfig};
    pub use crate::event::{Event, EventTrait, PipelineTrait, ProcessorTrait};
    pub use crate::event::{SimplePipeline, PipelineError};
    pub use crate::input::{StdinInput, RandomInput, Decoder, JsonDecoder, PlainDecoder, LineDecoder};
    pub use crate::output::{StdoutOutput, OutputFormat, JsonFormatter, PlainFormatter, PrettyFormatter};
    pub use crate::plugin::{Input, Filter, Output, Plugin, PluginType, PluginConfig};
    pub use crate::plugin::{PluginError, PluginResult, PluginRegistry, PluginManager, PluginFactory};
    pub use crate::plugin::traits::{InputStats, FilterStats, OutputStats};
    pub use crate::watcher::ConfigWatcher;
}