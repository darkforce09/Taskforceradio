//! The TeamSpeak 3 plugin for the 1.0 mod, built as a native Linux shared library.
//!
//! **Role:** the TeamSpeak plugin for the 1.0 mod: the `ts3plugin_*` exports, the game link
//! listener with its async queue, the TeamSpeak event handlers, antennas, `SETCFG`, the focus-aware
//! channel move and the debug window, wired to the 1.0 crates. Today it is the stub that proves
//! the path: the exports with the original metadata, the game link server on the 1.0 port and a
//! handler that answers `TS_INFO\tPING` with `PONG`.
//!
//! **Position:** `apps`, tier 6, depending on `tfar_ts3_host`, `tfar_ts3_settings_db`,
//! `tfar_game_link_server`, `tfar_web_requests`, `tfar_voice_pipeline_1_0`,
//! `tfar_radio_routing_1_0`, `tfar_client_state_1_0`, `tfar_game_commands_1_0`,
//! `tfar_plugin_commands_1_0`, `tfar_playback_mixer`, `tfar_geometry`, `tfar_arma_values`,
//! `tfar_plugin_settings_1_0`, `tfar_antennas_1_0`, `tfar_debug_window`; TeamSpeak 3 loads it from
//! its plugins folder.
//!
//! **Signals and state:** the plugin instance TeamSpeak loads ([`PLUGIN`], in [`plugin_exports`]), the game link
//! server's threads (accept, session and async worker) and the log forwarder thread, all started
//! in `ts3plugin_init` and joined in `ts3plugin_shutdown`.
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
