//! The plugin side of the game link: a localhost TCP listener that answers the extension.
//!
//! **Role:** the plugin side of the game link: listens on `127.0.0.1`, accepts the extension's
//! connection, hands each sync request to the plugin and writes its answer, and queues async
//! messages in arrival order.
//!
//! **Position:** `crates/game_link`, tier 1, depending on `tfar_game_link_protocol`; used by
//! `tfar_plugin_0_9`, `tfar_plugin_1_0`.
//!
//! **Signals and state:** the listener thread and the connection it serves.
//!
//! **Invariants:** async messages are delivered first in, first out.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
