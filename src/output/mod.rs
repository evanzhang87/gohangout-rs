//! Output plugins for GoHangout-rs
//!
//! This module contains output plugin implementations for writing data
//! to various destinations.

mod stdout;
mod format;

pub use stdout::StdoutOutput;
pub use format::{OutputFormat, JsonFormatter, PlainFormatter, PrettyFormatter, FormatError};

/// Re-exports for convenience
pub mod prelude {
    pub use super::{StdoutOutput, OutputFormat, JsonFormatter, PlainFormatter, PrettyFormatter};
}

/// Register built-in output plugins with the plugin factory
pub fn register_plugins(factory: &mut crate::plugin::PluginFactory) {
    factory.register_output("stdout", || {
        Ok(Box::new(StdoutOutput::default()) as Box<dyn crate::plugin::Output>)
    });
}

/// Create a default output plugin factory with all built-in plugins registered
pub fn default_factory() -> crate::plugin::PluginFactory {
    let mut factory = crate::plugin::PluginFactory::new();
    register_plugins(&mut factory);
    factory
}