//! The per-client state of the 0.9.12 plugin: positions, radios, tangents and effect state.
//!
//! **Role:** the state the 0.9.12 plugin keeps per TeamSpeak client, keyed by real name: position,
//! radios, tangent state, voice volume and the effect and HRTF state of each sender.
//!
//! **Position:** `crates/version_0_9`, tier 3, depending on `tfar_geometry`,
//! `tfar_game_commands_0_9`, `tfar_radio_effects_0_9`, `tfar_dsp_filters`, `tfar_dynamics`,
//! `tfar_hrtf`; used by `tfar_plugin_0_9`, `tfar_radio_routing_0_9`, `tfar_voice_pipeline_0_9`.
//!
//! **Signals and state:** the registry of clients behind one lock; effect state lives with its
//! client.
//!
//! **Invariants:** effect state is only reached while its client's lock is held.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
