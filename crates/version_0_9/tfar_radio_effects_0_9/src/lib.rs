//! The 0.9.12 radio sound chains: filters and noise per radio type.
//!
//! **Role:** the 0.9.12 radio effect chains: the filter and distortion settings per radio type and
//! the error level that degrades a signal with distance, with the original sign error fixed.
//!
//! **Position:** `crates/version_0_9`, tier 2, depending on `tfar_dsp_filters`,
//! `tfar_sample_buffer`; used by `tfar_client_state_0_9`, `tfar_voice_pipeline_0_9`.
//!
//! **Signals and state:** the filter state inside each `RadioEffect`.
//!
//! **Invariants:** the error level rises with distance, never falls.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
