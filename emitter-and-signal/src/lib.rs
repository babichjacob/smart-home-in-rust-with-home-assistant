pub mod emitter;
mod emitter_ext;
pub mod signal;
mod signal_ext;

pub use emitter_ext::EmitterExt;
pub use signal_ext::SignalExt;

#[derive(Debug, Clone, Copy)]
pub struct ProducerExited;
