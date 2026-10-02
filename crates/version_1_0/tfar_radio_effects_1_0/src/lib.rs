//! The 1.0 radio sound chains: filters, compressor input and noise per radio type.
//!
//! **Role:** the 1.0 radio effect chains: the filter settings per radio type and the error level
//! that degrades a signal with distance, with the original slope fixed.
//!
//! **Position:** `crates/version_1_0`, tier 2, depending on `tfar_dsp_filters`,
//! `tfar_sample_buffer`; used by `tfar_client_state_1_0`, `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** the filter state inside each `RadioEffect`.
//!
//! **Invariants:** the error level rises with distance, never falls.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
