//! The `task_force_radio_pipe` Arma extension for the 1.0 mod, a Windows x64 DLL run under Proton.
//!
//! **Role:** the Arma extension for the 1.0 mod: `~`-prefixed calls go async, data frames answer
//! `NEEDCFG` or `OK`, and `MISSIONEND` and `FOCUS` frames reach the 1.0 plugin over the game link.
//!
//! **Position:** `apps`, tier 2, depending on `tfar_arma_extension_abi`, `tfar_game_link_client`;
//! Arma 3 loads it through `callExtension` under Proton.
//!
//! **Signals and state:** the game link client the DLL keeps open.
//!
//! **Invariants:** async calls reach the plugin in the order the game made them.

#![allow(
    unsafe_code,
    reason = "the C exports Arma calls live in this crate; each `unsafe` block carries a `// SAFETY:` comment"
)]

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
