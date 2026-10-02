//! The distance curve and emitter cone gains that replace X3DAudio, plus the 0.9 stereo fallback.
//!
//! **Role:** the gain model that replaces X3DAudio: the piecewise distance curve, the emitter cone
//! gain and the simple interaural level difference the 0.9 stereo fallback uses.
//!
//! **Position:** `crates/audio`, tier 1, depending on `tfar_geometry`, `tfar_sample_buffer`; used
//! by `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** none; pure gain functions.
//!
//! **Invariants:** every gain is finite and within 0 to 1.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
