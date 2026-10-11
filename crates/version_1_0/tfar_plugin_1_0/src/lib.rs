//! The TeamSpeak 3 plugin for the 1.0 mod, built as a native Linux shared library.
//!
//! **Role:** the TeamSpeak plugin for the 1.0 mod: the `ts3plugin_*` exports, the game link
//! listener with its async queue, the TeamSpeak event handlers, antennas, `SETCFG`, the
//! focus-aware channel move, push-to-talk, the radio sounds, the voice callbacks and the debug
//! window, wired to the 1.0 crates. Every domain decision lives in a library crate; this crate is
//! wiring, threads and TeamSpeak side effects.
//!
//! **Position:** `crates/version_1_0`, tier 3, depending on `tfar_ts3_host`,
//! `tfar_ts3_settings_db`, `tfar_game_link::server`, `tfar_web_requests`, the other `_1_0` crates,
//! `tfar_audio::playback_mixer`, `tfar_audio::hrtf`, `tfar_audio::sample_buffer`, `tfar_foundation::arma_values` and
//! `tfar_debug_window`; TeamSpeak 3 loads it from its plugins folder.
//!
//! **Signals and state:** the plugin instance TeamSpeak loads ([`PLUGIN`], in [`plugin_exports`])
//! holding the [`runtime::PluginRuntime`]; the threads it starts in `ts3plugin_init` (the game
//! link server's two accept threads, session and async worker, `tfar-service`,
//! `tfar-tangent-release`, the debug window's UI thread, the update check and tracking requests,
//! the log forwarder) and joins in `ts3plugin_shutdown`, within 2 s. The lock order is in
//! `runtime::plugin_runtime`.
//!
//! **Invariants:** a plugin is dormant until its first game message and again after each game end
//! until the next one: no metadata, nickname, channel, mute or plugin command, and audio passes
//! through untouched; no TeamSpeak call is made while a server lock is held; the audio callbacks
//! never block.

// The `ts3plugin_*` exports TeamSpeak calls are written in this crate by `ts3_plugin!`.
#![allow(unsafe_code)]

pub mod audio;
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

pub use error::{Error, Result};
pub use plugin_exports::PLUGIN;
pub use plugin_lifecycle::TfarPlugin;
