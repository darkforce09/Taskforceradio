//! The 0.9.12 plugin-to-plugin commands sent through TeamSpeak, byte-compatible with the original.
//!
//! **Role:** the 0.9.12 plugin commands exchanged between TeamSpeak clients and the metadata block,
//! encoded byte for byte like the original so Windows players still interoperate.
//!
//! **Position:** `crates/version_0_9`, tier 2, depending on `tfar_arma_values`; used by
//! `tfar_plugin_0_9`.
//!
//! **Signals and state:** none; pure encoding.
//!
//! **Invariants:** the encoded bytes match what the original 0.9.12 plugin sends.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
