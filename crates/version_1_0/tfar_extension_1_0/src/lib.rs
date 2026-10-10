//! The `task_force_radio_pipe` Arma extension for the 1.0 mod, a Windows x64 DLL run under Proton.
//!
//! **Role:** the Arma extension for the 1.0 mod: `~`-suffixed calls go async, data frames answer
//! `NEEDCFG` or `OK`, and `MISSIONEND` and `FOCUS` frames reach the 1.0 plugin over the game link.
//! [`extension_exports`] writes the `RVExtension` and `RVExtensionVersion` exports and keeps the
//! process-wide game link client and [`FocusWatch`]; [`call_handling`] maps each call to the reply
//! string of the original extension.
//!
//! **Position:** `crates/version_1_0`, tier 2, depending on `tfar_arma_extension_abi`,
//! `tfar_game_link_client`; Arma 3 loads it through `callExtension` under Proton.
//!
//! **Signals and state:** the game link client the DLL keeps open, created on the first call in a
//! `OnceLock` (its port is the 1.0 plugin's, or the debug port when Arma's command line holds
//! `-tfdebug`), and the focus watch behind a mutex, both used from Arma's calling thread.
//!
//! **Invariants:** async calls reach the plugin in the order the game made them and are answered
//! without waiting for it; every call returns within the sync timeout, connected or not (about
//! 1.3 s when it also connects); the foreground window is read at most every 250 ms.

// The `RVExtension` and `RVExtensionVersion` exports Arma calls are written in this crate.
#![allow(unsafe_code)]

pub mod call_handling;
mod crate_role;
mod error;
pub mod extension_exports;
mod focus_watch;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
pub use extension_exports::{DEBUG_SWITCH, EXTENSION_VERSION, FLAVOUR, client_config};
pub use focus_watch::{FOCUS_READ_INTERVAL, FocusWatch};
