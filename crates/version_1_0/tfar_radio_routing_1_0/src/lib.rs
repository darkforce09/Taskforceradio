//! Whether and how a 1.0 listener hears a speaker: direct, over radio, intercom or speaker radio.
//!
//! **Role:** the 1.0 reception rules: `isOverRadio` and its neighbours decide whether a listener
//! hears a speaker directly, over radio (with antennas and terrain interception), over intercom or
//! through a speaker radio, and with which effect.
//!
//! **Position:** `crates/version_1_0`, tier 4, depending on `tfar_client_state_1_0`,
//! `tfar_antennas_1_0`, `tfar_plugin_settings_1_0`, `tfar_geometry`; used by `tfar_plugin_1_0`,
//! `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** none; it reads client state and antennas.
//!
//! **Invariants:** routing reads client state and never changes it.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
