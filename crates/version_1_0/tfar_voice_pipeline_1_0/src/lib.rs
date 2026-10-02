//! The 1.0 audio path: one TeamSpeak voice frame through routing, effects, compressor, HRTF and
//! mixing.
//!
//! **Role:** the 1.0 audio path for one voice frame: routes the speaker, applies the radio effect
//! and the compressor once, the distance and cone gains and the HRTF, and mixes radio sounds.
//!
//! **Position:** `crates/version_1_0`, tier 5, depending on `tfar_client_state_1_0`,
//! `tfar_radio_routing_1_0`, `tfar_radio_effects_1_0`, `tfar_plugin_settings_1_0`,
//! `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `tfar_spatial_audio`, `tfar_sample_buffer`,
//! `tfar_playback_mixer`; used by `tfar_plugin_1_0`.
//!
//! **Signals and state:** none of its own; it works on client state and the buffer it is handed.
//!
//! **Invariants:** the compressor runs once per frame.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
