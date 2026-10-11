//! The names a caller of `tfar_audio::dynamics` imports with `use crate::dynamics::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::dynamics::compressor::{Compressor, DC_OFFSET};
pub use crate::dynamics::compressor_settings::CompressorSettings;
