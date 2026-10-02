//! Parsing of the strings Arma sends: tab-split fields, Arma arrays, frequency lists, positions.
//!
//! **Role:** the parsers for every value shape Arma's `callExtension` strings carry: tab-split
//! fields split exactly like the original getline loop, Arma array literals, frequency lists and
//! positions.
//!
//! **Position:** `crates/foundation`, tier 1, depending on `tfar_geometry`; used by
//! `tfar_antennas_1_0`, `tfar_game_commands_0_9`, `tfar_game_commands_1_0`, `tfar_plugin_0_9`,
//! `tfar_plugin_1_0`, `tfar_plugin_commands_0_9`, `tfar_plugin_commands_1_0`.
//!
//! **Signals and state:** none; pure parsing.
//!
//! **Invariants:** a parser never indexes past the tokens it was given; a short message is an
//! `Error`, never a panic.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
