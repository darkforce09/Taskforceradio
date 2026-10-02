//! The feed-forward compressor the radio effects run once per frame.
//!
//! **Role:** the feed-forward compressor that replaces SimpleComp: threshold, ratio, attack and
//! release over a sample buffer, run exactly once per frame.
//!
//! **Position:** `crates/audio`, tier 1, depending on `tfar_sample_buffer`; used by
//! `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_voice_pipeline_0_9`,
//! `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** the envelope follower state inside each `Compressor`.
//!
//! **Invariants:** one `process` call is one pass over the frame; the caller never runs it twice.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
