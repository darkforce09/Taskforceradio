//! Binaural direction rendering over the IRCAM HRIR sphere through the hrtf crate.
//!
//! **Role:** real binaural direction rendering for both plugin versions: loads the IRCAM HRIR
//! sphere through the `hrtf` crate and adapts TeamSpeak's frame sizes to its block size, keeping
//! state per sound source.
//!
//! **Position:** a module of `tfar_audio`, depending on `tfar_foundation::geometry`, `tfar_audio::sample_buffer`; used
//! by `tfar_plugin_logic_0_9::client_state`, `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_0_9::voice_pipeline`,
//! `tfar_plugin_logic_1_0::voice_pipeline`.
//!
//! **Signals and state:** [`HrirLibrary`] is loaded once off the audio thread and shared read-only
//! (`Arc`); each [`HrtfSource`] owns its copy of the prepared sphere, its convolution tails and its
//! block buffers, and is used from the TeamSpeak audio thread without allocating.
//!
//! **Invariants:** a source's state belongs to one speaker and is never shared between two; the
//! output lags the input by exactly [`ADDED_LATENCY_FRAMES`] (480 frames, 10 ms) whatever the frame
//! sizes; no sphere reaches the `hrtf` crate without passing the structural check.

mod block_adapter;
mod error;
mod hrir_library;
mod hrtf_source;
mod listener_relative_direction;
pub mod prelude;
mod render_timing;
mod sphere_file_check;

#[cfg(test)]
#[path = "tests/real_sphere.rs"]
mod real_sphere;

pub use error::{Error, Result};
pub use hrir_library::HrirLibrary;
pub use hrtf_source::HrtfSource;
pub use listener_relative_direction::ListenerRelativeDirection;
pub use render_timing::{
    ADDED_LATENCY_FRAMES, CONVOLUTION_BLOCK_FRAMES, IDLE_RESET_AFTER, INTERPOLATION_STEPS,
    RENDER_BLOCK_FRAMES, SAMPLE_RATE_HZ,
};
