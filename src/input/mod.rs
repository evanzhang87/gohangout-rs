//! Input plugins for GoHangout-rs
//!
//! This module contains input plugin implementations for reading data
//! from various sources.

mod stdin;
mod codec;
mod random;

pub use stdin::StdinInput;
pub use codec::{Decoder, JsonDecoder, PlainDecoder, LineDecoder, DecoderError};
pub use random::{RandomInput, RandomMode, FieldType};

/// Re-exports for convenience
pub mod prelude {
    pub use super::{StdinInput, RandomInput, Decoder, JsonDecoder, PlainDecoder, LineDecoder};
}

/// Register built-in input plugins with the plugin factory
pub fn register_plugins(factory: &mut crate::plugin::PluginFactory) {
    factory.register_input("stdin", || {
        Ok(Box::new(StdinInput::default()) as Box<dyn crate::plugin::Input>)
    });
    
    factory.register_input("random", || {
        Ok(Box::new(RandomInput::default()) as Box<dyn crate::plugin::Input>)
    });
}

/// Create a default input plugin factory with all built-in plugins registered
pub fn default_factory() -> crate::plugin::PluginFactory {
    let mut factory = crate::plugin::PluginFactory::new();
    register_plugins(&mut factory);
    factory
}