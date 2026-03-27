//! Event module for GoHangout-rs
//!
//! This module defines the core event structure and processing pipeline
//! for the ETL system.

mod event;
mod traits;
mod pipeline;
mod processor;

pub use event::Event;
pub use traits::{EventTrait, PipelineTrait, ProcessorTrait};
pub use pipeline::{SimplePipeline, PipelineError};
pub use processor::SimpleProcessor;

/// Re-exports for convenience
pub mod prelude {
    pub use super::{Event, EventTrait, PipelineTrait, ProcessorTrait};
    pub use super::{SimplePipeline, PipelineError};
}