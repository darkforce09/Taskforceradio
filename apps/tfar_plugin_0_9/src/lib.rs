//! The TeamSpeak 3 plugin for the 0.9.12 mod, built as a native Linux shared library.
//!
//! **Role:** the TeamSpeak plugin for the 0.9.12 mod: the `ts3plugin_*` exports, the game link
//! listener, the TeamSpeak event handlers, push-to-talk and voice activation, metadata and serious
//! mode, wired to the 0.9 crates. Today it is the stub that proves the path: the exports with the
//! original metadata, the game link server on the 0.9 port and a handler that answers
//! `TS_INFO\tPING` with `PONG`.
//!
//! **Position:** `apps`, tier 6, depending on `tfar_ts3_host`, `tfar_ts3_settings_db`,
//! `tfar_game_link_server`, `tfar_web_requests`, `tfar_voice_pipeline_0_9`,
//! `tfar_radio_routing_0_9`, `tfar_client_state_0_9`, `tfar_game_commands_0_9`,
//! `tfar_plugin_commands_0_9`, `tfar_playback_mixer`, `tfar_geometry`, `tfar_arma_values`;
//! TeamSpeak 3 loads it from its plugins folder.
//!
//! **Signals and state:** the plugin instance TeamSpeak loads ([`PLUGIN`], in [`plugin_exports`]), the game link
//! server's threads and the log forwarder thread, all started in `ts3plugin_init` and joined in
//! `ts3plugin_shutdown`.
//!
//! **Invariants:** a plugin whose game link never connected this session stays dormant and passes
//! audio through untouched; no thread outlives `ts3plugin_shutdown`.

// The `ts3plugin_*` exports TeamSpeak calls are written in this crate by `ts3_plugin!`.
#![allow(unsafe_code)]

mod crate_role;
mod error;
pub mod game_link_stub;
pub mod listen_port;
pub mod plugin_exports;
pub mod plugin_lifecycle;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
pub use plugin_exports::PLUGIN;
pub use plugin_lifecycle::TfarPlugin;
