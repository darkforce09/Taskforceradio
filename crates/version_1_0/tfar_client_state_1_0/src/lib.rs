//! The per-client state of the 1.0 plugin: positions, radios, tangents, vehicles and effect state.
//!
//! **Role:** the state the 1.0 plugin keeps per TeamSpeak client, keyed by real name: position,
//! radios, tangent state, vehicle and intercom, and the effect and HRTF state of each sender.
//!
//! **Position:** `crates/version_1_0`, tier 3, depending on `tfar_geometry`,
//! `tfar_game_commands_1_0`, `tfar_radio_effects_1_0`, `tfar_plugin_settings_1_0`,
//! `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`; used by `tfar_plugin_1_0`,
//! `tfar_radio_routing_1_0`, `tfar_voice_pipeline_1_0`.
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
