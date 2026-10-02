//! The safe layer over the TeamSpeak 3 plugin SDK (API 26): exports, callbacks, function table,
//! logging.
//!
//! **Role:** the boundary to TeamSpeak: the `ts3plugin_*` export layer over `ts3plugin-sys`, a safe
//! wrapper around the `Ts3Functions` table, C-string ownership, and a `tracing` layer that forwards
//! to TeamSpeak's log.
//!
//! **Position:** `crates/teamspeak`, tier 0, depending on no workspace crate; used by
//! `tfar_plugin_0_9`, `tfar_plugin_1_0`.
//!
//! **Signals and state:** the function table handed over at load time; no other global state.
//!
//! **Invariants:** no panic or unwind crosses into TeamSpeak; every C string handed over stays
//! alive for the call.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
