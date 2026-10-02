//! The TeamSpeak 3 plugin for the 0.9.12 mod, built as a native Linux shared library.
//!
//! **Role:** the TeamSpeak plugin for the 0.9.12 mod: the `ts3plugin_*` exports, the game link
//! listener, the TeamSpeak event handlers, push-to-talk and voice activation, metadata and serious
//! mode, wired to the 0.9 crates.
//!
//! **Position:** `apps`, tier 6, depending on `tfar_ts3_host`, `tfar_ts3_settings_db`,
//! `tfar_game_link_server`, `tfar_web_requests`, `tfar_voice_pipeline_0_9`,
//! `tfar_radio_routing_0_9`, `tfar_client_state_0_9`, `tfar_game_commands_0_9`,
//! `tfar_plugin_commands_0_9`, `tfar_playback_mixer`, `tfar_geometry`, `tfar_arma_values`;
//! TeamSpeak 3 loads it from its plugins folder.
//!
//! **Signals and state:** the plugin instance TeamSpeak loads, its worker threads and the game link
//! listener.
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
