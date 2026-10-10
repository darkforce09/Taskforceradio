//! The `task_force_radio_pipe` Arma extension for the 0.9.12 mod, a Windows x64 DLL run under
//! Proton.
//!
//! **Role:** the Arma extension for the 0.9.12 mod: every `callExtension` is a sync round trip to
//! the 0.9 plugin over the game link. [`extension_exports`] writes the `RVExtension` and
//! `RVExtensionVersion` exports and keeps the process-wide game link client; [`call_handling`]
//! maps each outcome to the reply string of the original extension.
//!
//! **Position:** `crates/version_0_9`, tier 2, depending on `tfar_arma_extension_abi`,
//! `tfar_game_link_client`; Arma 3 loads it through `callExtension` under Proton.
//!
//! **Signals and state:** the game link client the DLL keeps open, created on the first call in a
//! `OnceLock`; its port is the 0.9 plugin's, or the debug port when Arma's command line holds
//! `-tfdebug`.
//!
//! **Invariants:** every call returns within the sync timeout, connected or not (about 1.3 s when
//! it also connects); empty input replies empty and sends nothing.

// The `RVExtension` and `RVExtensionVersion` exports Arma calls are written in this crate.
#![allow(unsafe_code)]

pub mod call_handling;
mod crate_role;
mod error;
pub mod extension_exports;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
pub use extension_exports::{DEBUG_SWITCH, EXTENSION_VERSION, FLAVOUR, client_config};
