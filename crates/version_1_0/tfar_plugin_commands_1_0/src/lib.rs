//! The 1.0 plugin-to-plugin commands sent through TeamSpeak, byte-compatible with the original.
//!
//! **Role:** the 1.0 plugin commands exchanged between TeamSpeak clients and the metadata block,
//! encoded byte for byte like the original so Windows players still interoperate.
//!
//! **Position:** `crates/version_1_0`, tier 2, depending on `tfar_arma_values`; used by
//! `tfar_plugin_1_0`.
//!
//! **Signals and state:** none; pure encoding.
//!
//! **Invariants:** the encoded bytes match what the original 1.0 plugin sends.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
