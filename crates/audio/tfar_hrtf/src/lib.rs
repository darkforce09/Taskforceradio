//! Binaural direction rendering over the IRCAM HRIR sphere through the hrtf crate.
//!
//! **Role:** real binaural direction rendering for both plugin versions: loads the IRCAM HRIR
//! sphere through the `hrtf` crate and adapts TeamSpeak's frame sizes to its block size, keeping
//! state per sound source.
//!
//! **Position:** `crates/audio`, tier 1, depending on `tfar_geometry`, `tfar_sample_buffer`; used
//! by `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_voice_pipeline_0_9`,
//! `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** per-source convolution state; the sphere is shared read-only.
//!
//! **Invariants:** a source's state belongs to one speaker and is never shared between two.

mod crate_role;
mod error;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
