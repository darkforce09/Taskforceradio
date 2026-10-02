//! Interleaved 16-bit sample frames as TeamSpeak hands them out, with saturating gain.
//!
//! **Role:** the sample frame type the audio crates pass around: interleaved `i16` channels as
//! TeamSpeak delivers them, conversion to and from `f32`/`f64`, and gain that saturates instead of
//! wrapping.
//!
//! **Position:** `crates/audio`, tier 0, depending on no workspace crate; used by
//! `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `tfar_playback_mixer`,
//! `tfar_radio_effects_0_9`, `tfar_radio_effects_1_0`, `tfar_spatial_audio`,
//! `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** none; buffers are borrowed from the caller.
//!
//! **Invariants:** gain saturates at the `i16` limits and never wraps.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
