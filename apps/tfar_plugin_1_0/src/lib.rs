//! The TeamSpeak 3 plugin for the 1.0 mod, built as a native Linux shared library.
//!
//! **Role:** the TeamSpeak plugin for the 1.0 mod: the `ts3plugin_*` exports, the game link
//! listener with its async queue, the TeamSpeak event handlers, antennas, `SETCFG`, the focus-aware
//! channel move and the debug window, wired to the 1.0 crates.
//!
//! **Position:** `apps`, tier 6, depending on `tfar_ts3_host`, `tfar_ts3_settings_db`,
//! `tfar_game_link_server`, `tfar_web_requests`, `tfar_voice_pipeline_1_0`,
//! `tfar_radio_routing_1_0`, `tfar_client_state_1_0`, `tfar_game_commands_1_0`,
//! `tfar_plugin_commands_1_0`, `tfar_playback_mixer`, `tfar_geometry`, `tfar_arma_values`,
//! `tfar_plugin_settings_1_0`, `tfar_antennas_1_0`, `tfar_debug_window`; TeamSpeak 3 loads it from
//! its plugins folder.
//!
//! **Signals and state:** the plugin instance TeamSpeak loads, its worker threads, the game link
//! listener and the debug window thread.
//!
//! **Invariants:** a plugin whose game link never connected this session stays dormant and passes
//! audio through untouched.

#![allow(
    unsafe_code,
    reason = "the C exports TeamSpeak calls live in this crate; each `unsafe` block carries a `// SAFETY:` comment"
)]

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
