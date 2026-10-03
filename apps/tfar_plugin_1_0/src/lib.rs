//! The TeamSpeak 3 plugin for the 1.0 mod, built as a native Linux shared library.
//!
//! **Role:** the TeamSpeak plugin for the 1.0 mod: the `ts3plugin_*` exports, the game link
//! listener with its async queue, the TeamSpeak event handlers, antennas, `SETCFG`, the
//! focus-aware channel move, push-to-talk, the radio sounds, the voice callbacks and the debug
//! window, wired to the 1.0 crates. Every domain decision lives in a library crate; this crate is
//! wiring, threads and TeamSpeak side effects.
//!
//! **Position:** `apps`, tier 6, depending on `tfar_ts3_host`, `tfar_ts3_settings_db`,
//! `tfar_game_link_server`, `tfar_web_requests`, the `_1_0` crates, `tfar_playback_mixer`,
//! `tfar_hrtf`, `tfar_sample_buffer`, `tfar_dsp_filters`, `tfar_spatial_audio`, `tfar_geometry`,
//! `tfar_arma_values` and `tfar_debug_window`; TeamSpeak 3 loads it from its plugins folder.
//!
//! **Signals and state:** the plugin instance TeamSpeak loads ([`PLUGIN`], in [`plugin_exports`])
//! holding the [`runtime::PluginRuntime`]; the threads it starts in `ts3plugin_init` (the game
//! link server's accept, session and async worker, `tfar-service`, `tfar-tangent-release`, the
//! debug window's UI thread, the update check, the log forwarder) and joins in
//! `ts3plugin_shutdown`. The lock order is in `runtime::plugin_runtime`.
//!
//! **Invariants:** a plugin whose game link never carried a game message this session stays
//! dormant: no metadata, nickname, channel, mute or plugin command, and audio passes through
//! untouched; no TeamSpeak call is made while a server lock is held; the audio callbacks never
//! block.

// The `ts3plugin_*` exports TeamSpeak calls are written in this crate by `ts3_plugin!`.
#![allow(unsafe_code)]

pub mod audio;
mod crate_role;
pub mod diagnostics;
mod error;
pub mod events;
pub mod game_link;
pub mod listen_port;
pub mod plugin_exports;
pub mod plugin_lifecycle;
pub mod prelude;
pub mod push_to_talk;
pub mod runtime;
pub mod service;
pub mod teamspeak_actions;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
pub use plugin_exports::PLUGIN;
pub use plugin_lifecycle::TfarPlugin;
