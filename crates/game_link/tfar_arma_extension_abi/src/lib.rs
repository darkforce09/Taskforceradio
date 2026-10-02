//! The `RVExtension` exports and the Win32 calls an Arma extension DLL needs.
//!
//! **Role:** the Arma side of an extension DLL: the `RVExtension` and `RVExtensionVersion` exports,
//! output writes truncated to Arma's buffer size, and the focus query through
//! `GetForegroundWindow`.
//!
//! **Position:** `crates/game_link`, tier 0, depending on no workspace crate; used by
//! `tfar_extension_0_9`, `tfar_extension_1_0`.
//!
//! **Signals and state:** none beyond what the exporting app holds.
//!
//! **Invariants:** a reply never writes past the buffer Arma passes.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
