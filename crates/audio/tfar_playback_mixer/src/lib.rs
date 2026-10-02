//! Loading and mixing of the radio sound WAV files into the outgoing audio.
//!
//! **Role:** the radio sound player: loads the WAV files shipped in the mod's `radio-sounds` folder
//! through `hound`, checks they are 48 kHz 16-bit stereo, caches them and mixes active playbacks
//! into a frame.
//!
//! **Position:** `crates/audio`, tier 1, depending on `tfar_sample_buffer`; used by
//! `tfar_plugin_0_9`, `tfar_plugin_1_0`, `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** the sound cache and the list of active playbacks.
//!
//! **Invariants:** a playback's filters and buffers are freed when it ends.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
