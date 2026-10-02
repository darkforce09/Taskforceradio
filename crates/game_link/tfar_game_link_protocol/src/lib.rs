//! The frames the Arma extensions and the TeamSpeak plugins exchange over localhost TCP.
//!
//! **Role:** the wire format of the game link: `u32` little-endian length, a kind byte and a
//! `postcard` payload, the `HELLO` handshake, the frame kinds and the ports each plugin listens on.
//!
//! **Position:** `crates/game_link`, tier 0, depending on no workspace crate; used by
//! `tfar_game_link_client`, `tfar_game_link_server`, `tfar_ts3_mock`.
//!
//! **Signals and state:** none; pure encoding.
//!
//! **Invariants:** a frame decodes back to the frame that was encoded, and an oversized length is
//! refused.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
