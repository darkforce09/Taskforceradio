//! The 1.0 plugin's settings: the values `SETCFG` sets and the defaults.
//!
//! **Role:** the 1.0 plugin's settings model: every value the game sets with `SETCFG`, its default
//! and its type.
//!
//! **Position:** `crates/version_1_0`, tier 0, depending on no workspace crate; used by
//! `tfar_client_state_1_0`, `tfar_game_commands_1_0`, `tfar_plugin_1_0`, `tfar_radio_routing_1_0`,
//! `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** none; a value type the plugin owns.
//!
//! **Invariants:** an unknown key or a malformed value is an `Error` and leaves the settings
//! unchanged.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
