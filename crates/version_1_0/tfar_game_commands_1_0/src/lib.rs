//! The 1.0 messages Arma sends the plugin, parsed into typed commands.
//!
//! **Role:** the typed form of every 1.0 game message (`POS`, `TANGENT`, `FREQ`, `SETCFG`,
//! `MISSIONEND`, data frames and the rest), parsed with checked token counts.
//!
//! **Position:** `crates/version_1_0`, tier 2, depending on `tfar_arma_values`, `tfar_geometry`,
//! `tfar_plugin_settings_1_0`; used by `tfar_client_state_1_0`, `tfar_plugin_1_0`.
//!
//! **Signals and state:** none; pure parsing.
//!
//! **Invariants:** `TANGENT` parses by its own token layout, never by shifted indices.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
