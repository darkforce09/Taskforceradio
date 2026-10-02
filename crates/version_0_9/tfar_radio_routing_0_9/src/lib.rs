//! Whether and how a 0.9.12 listener hears a speaker: direct, over radio or over a speaker radio.
//!
//! **Role:** the 0.9.12 reception rules: `isOverLocalRadio` and `isOverRadio` decide whether a
//! listener hears a speaker directly, over SW, LR or DD radio or through a speaker radio, and with
//! which effect.
//!
//! **Position:** `crates/version_0_9`, tier 4, depending on `tfar_client_state_0_9`,
//! `tfar_geometry`; used by `tfar_plugin_0_9`, `tfar_voice_pipeline_0_9`.
//!
//! **Signals and state:** none; it reads client state.
//!
//! **Invariants:** routing reads client state and never changes it.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
