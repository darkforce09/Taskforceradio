//! The feed-forward compressor the radio effects run once per frame.
//!
//! **Role:** the feed-forward compressor that replaces SimpleComp: threshold, ratio, attack and
//! release over a sample buffer, run exactly once per frame.
//!
//! **Position:** a module of `tfar_audio`, depending on `tfar_audio::sample_buffer`; used by
//! `tfar_plugin_logic_0_9::client_state`, `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_0_9::voice_pipeline`,
//! `tfar_plugin_logic_1_0::voice_pipeline`.
//!
//! **Signals and state:** the envelope follower state inside each [`Compressor`]; no globals.
//! Built off the TeamSpeak audio thread, run on it; processing allocates nothing.
//!
//! **Invariants:** one `process` call is one pass over the frame; the caller never runs it twice.
//! The threshold is on the raw `i16` sample scale the originals compress on.

mod attack_release_envelope;
mod compressor;
mod compressor_settings;
mod error;
pub mod prelude;

pub use compressor::{Compressor, DC_OFFSET};
pub use compressor_settings::CompressorSettings;
pub use error::{Error, Result};
