//! The `task_force_radio_pipe` Arma extension for the 0.9.12 mod, a Windows x64 DLL run under
//! Proton.
//!
//! **Role:** the Arma extension for the 0.9.12 mod: every `callExtension` is a sync round trip to
//! the 0.9 plugin over the game link.
//!
//! **Position:** `apps`, tier 2, depending on `tfar_arma_extension_abi`, `tfar_game_link_client`;
//! Arma 3 loads it through `callExtension` under Proton.
//!
//! **Signals and state:** the game link client the DLL keeps open.
//!
//! **Invariants:** every call returns within the sync timeout, connected or not.

#![allow(
    unsafe_code,
    reason = "the C exports Arma calls live in this crate; each `unsafe` block carries a `// SAFETY:` comment"
)]

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
