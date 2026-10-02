//! The feed-forward compressor the radio effects run once per frame.
//!
//! **Role:** the feed-forward compressor that replaces SimpleComp: threshold, ratio, attack and
//! release over a sample buffer, run exactly once per frame.
//!
//! **Position:** `crates/audio`, tier 1, depending on `tfar_sample_buffer`; used by
//! `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_voice_pipeline_0_9`,
//! `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** the envelope follower state inside each [`Compressor`]; no globals.
//! Built off the TeamSpeak audio thread, run on it; processing allocates nothing.
//!
//! **Invariants:** one `process` call is one pass over the frame; the caller never runs it twice.
//! The threshold is on the raw `i16` sample scale the originals compress on.

mod attack_release_envelope;
mod compressor;
mod compressor_settings;
mod crate_role;
mod error;
pub mod prelude;

pub use compressor::{Compressor, DC_OFFSET};
pub use compressor_settings::CompressorSettings;
pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
