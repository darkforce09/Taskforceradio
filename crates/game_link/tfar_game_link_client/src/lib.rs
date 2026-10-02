//! The extension side of the game link: a lazy, time-bounded localhost TCP client.
//!
//! **Role:** the extension side of the game link: connects lazily with a connect timeout of at most
//! 50 ms and a 1 s backoff, sends a sync request and waits at most 1000 ms, and returns the
//! original game-facing strings when the plugin is not there.
//!
//! **Position:** `crates/game_link`, tier 1, depending on `tfar_game_link_protocol`; used by
//! `tfar_extension_0_9`, `tfar_extension_1_0`, `tfar_ts3_mock`.
//!
//! **Signals and state:** one connection, reopened lazily after a backoff.
//!
//! **Invariants:** a call never blocks the game for longer than the sync timeout.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
