//! The 0.9.12 audio path: one TeamSpeak voice frame through routing, effects, HRTF and mixing.
//!
//! **Role:** the 0.9.12 audio path for one voice frame: routes the speaker, applies the radio
//! effect, the distance and stereo gains and the HRTF, and mixes radio sounds.
//!
//! **Position:** `crates/version_0_9`, tier 5, depending on `tfar_client_state_0_9`,
//! `tfar_radio_routing_0_9`, `tfar_radio_effects_0_9`, `tfar_dsp_filters`, `tfar_dynamics`,
//! `tfar_hrtf`, `tfar_spatial_audio`, `tfar_sample_buffer`, `tfar_playback_mixer`; used by
//! `tfar_plugin_0_9`.
//!
//! **Signals and state:** none of its own; it works on client state and the buffer it is handed.
//!
//! **Invariants:** the post-process never reads past the frame it was handed.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
