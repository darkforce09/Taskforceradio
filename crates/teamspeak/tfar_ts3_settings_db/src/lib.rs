//! Reads TeamSpeak's settings.db for the values the plugins need.
//!
//! **Role:** the reader of TeamSpeak's `settings.db` SQLite file, for the hotkey and capture
//! settings the plugins look up.
//!
//! **Position:** `crates/teamspeak`, tier 0, depending on no workspace crate; used by
//! `tfar_plugin_0_9`, `tfar_plugin_1_0`.
//!
//! **Signals and state:** one read-only connection per open.
//!
//! **Invariants:** the database is never written.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
