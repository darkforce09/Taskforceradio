//! The safe layer over the TeamSpeak 3 plugin SDK (API 26): exports, callbacks, function table,
//! logging.
//!
//! **Role:** the boundary to TeamSpeak: the `ts3plugin_*` export layer ([`ts3_plugin!`] over the
//! [`Ts3Plugin`] trait), the safe [`Ts3`] handle over the `TS3Functions` table of `ts3plugin-sys`,
//! C-string and list ownership, and a `tracing` layer that forwards to TeamSpeak's log.
//!
//! **Position:** `crates/teamspeak`, tier 0, depending on no workspace crate; used by
//! `tfar_plugin_0_9`, `tfar_plugin_1_0`.
//!
//! **Signals and state:** the function table and plugin ID, set once on TeamSpeak's main thread
//! (`OnceLock`); the log queue and its logger thread. Audio callbacks arrive on TeamSpeak's audio
//! threads and touch neither.
//!
//! **Invariants:** no panic or unwind crosses into TeamSpeak; every C string handed over stays
//! alive for the call; every TeamSpeak allocation is copied and freed with its `freeMemory`; the
//! `ts3plugin-sys` layout is checked against the SDK headers in `sdk_headers/`.

// The crate is the FFI boundary to TeamSpeak: exports, the function table and C memory.
#![allow(unsafe_code)]

mod crate_role;
mod error;
mod host_ids;
mod host_values;
mod logging;
mod plugin_exports;
pub mod prelude;
mod ts3_functions;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result, Ts3Error};
pub use host_ids::{ChannelId, ClientId, ServerConnectionHandlerId};
pub use host_values::{
    ChannelProperty, ClientProperty, ConnectStatus, LogLevel, PluginCommandTarget, ServerProperty,
    Ts3Vector, Visibility,
};
pub use logging::{
    LoggingGuard, TeamSpeakLogLayer, install_logging, install_logging_at, print_to_current_tab,
};
#[doc(hidden)]
pub use plugin_exports::export_glue;
pub use plugin_exports::{
    CapturedVoiceData, ClientMove, CommandHandling, ErrorDisplay, InfoItem, InitStatus, Invoker,
    MixedPlaybackVoiceData, PluginCommand, PostProcessVoiceData, ServerError,
    ServerPermissionError, Ts3Plugin,
};
pub use ts3_functions::Ts3;

#[cfg(test)]
#[path = "tests/sdk_layout.rs"]
mod sdk_layout_tests;

#[cfg(test)]
#[path = "tests/fake_ts3_table.rs"]
mod fake_ts3_table;
