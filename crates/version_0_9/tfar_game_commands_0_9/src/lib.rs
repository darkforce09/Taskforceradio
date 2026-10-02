//! The 0.9.12 messages Arma sends the plugin, parsed into typed commands.
//!
//! **Role:** the typed form of every 0.9.12 game message (`POS`, `TANGENT`, `FREQ`, `KILLED`,
//! `TRACK` and the rest), parsed with checked token counts.
//!
//! **Position:** `crates/version_0_9`, tier 2, depending on `tfar_arma_values`, `tfar_geometry`;
//! used by `tfar_client_state_0_9`, `tfar_plugin_0_9`.
//!
//! **Signals and state:** none; pure parsing.
//!
//! **Invariants:** every token access is length-checked, so a short message is an `Error`.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
