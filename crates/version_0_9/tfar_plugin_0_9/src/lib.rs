//! The TeamSpeak 3 plugin for the 0.9.12 mod, built as a native Linux shared library.
//!
//! **Role:** the TeamSpeak plugin for the 0.9.12 mod: the `ts3plugin_*` exports, the game link
//! listener and its message handling, the TeamSpeak event handlers, the voice callbacks,
//! push-to-talk and voice activation, the radio sounds, metadata and serious mode, wired to the
//! 0.9 crates. Every domain decision lives in a library; this crate is wiring, threads and
//! TeamSpeak side effects.
//!
//! **Position:** `crates/version_0_9`, tier 6, depending on `tfar_ts3_host`,
//! `tfar_ts3_settings_db`, `tfar_game_link_server`, `tfar_web_requests`, `tfar_voice_pipeline_0_9`,
//! `tfar_radio_routing_0_9`, `tfar_client_state_0_9`, `tfar_game_commands_0_9`,
//! `tfar_plugin_commands_0_9`, `tfar_playback_mixer`, `tfar_geometry`, `tfar_arma_values`,
//! `tfar_hrtf`, `tfar_sample_buffer`, `tfar_dsp_filters`, `tfar_spatial_audio`; TeamSpeak 3
//! loads it from its plugins folder.
//!
//! **Signals and state:** the plugin instance TeamSpeak loads ([`PLUGIN`], in
//! [`plugin_exports`]); the shared [`plugin_core::PluginCore`]; the threads `init` starts and
//! `shutdown` joins: the game link server's, `tfar-service`, `tfar-push-to-talk-release`, the
//! web requests and the log forwarder; TeamSpeak's main and audio threads call in. Lock order:
//! push-to-talk, then the server map, then one connection's state; no TeamSpeak call under the
//! latter two.
//!
//! **Invariants:** a plugin with no game activity since the session began or since its last game
//! end is dormant: it writes nothing to TeamSpeak and passes audio through untouched; an audio
//! callback waits only for a connection state, whose holders never call TeamSpeak;
//! `ts3plugin_shutdown` returns within 2 s, and the only thread that may outlive it is a
//! cancelled web request still waiting for the network, which ends within its own timeout.

// The `ts3plugin_*` exports TeamSpeak calls are written in this crate by `ts3_plugin!`.
#![allow(unsafe_code)]

pub mod audio_callbacks;
pub mod background_web_requests;
pub mod channel_snapshot;
mod error;
pub mod game_messages;
pub mod game_session_transitions;
pub mod incoming_plugin_commands;
pub mod listen_port;
pub mod plugin_core;
pub mod plugin_exports;
pub mod plugin_lifecycle;
pub mod plugin_startup;
pub mod prelude;
pub mod push_to_talk;
pub mod radio_sounds;
pub mod running_plugin;
pub mod server_connection;
pub mod service_thread;
pub mod status_metadata;
pub mod team_speak;
pub mod team_speak_events;

#[cfg(test)]
#[path = "tests/fake_team_speak.rs"]
mod fake_team_speak;

pub use error::{Error, Result};
pub use plugin_exports::PLUGIN;
pub use plugin_lifecycle::TfarPlugin;
