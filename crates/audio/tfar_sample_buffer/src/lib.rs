//! Interleaved 16-bit sample frames as TeamSpeak hands them out, with saturating gain.
//!
//! **Role:** the sample helpers the audio crates share: the owned stereo buffer, bounds-checked
//! adapters between TeamSpeak's mono, stereo and multichannel blocks and that buffer, gain and
//! additive mixing that saturate instead of wrapping, and conversion to and from planar `f32`.
//!
//! **Position:** `crates/audio`, tier 0, depending on no workspace crate; used by
//! `tfar_dsp_filters`, `tfar_dynamics`, `tfar_hrtf`, `tfar_playback_mixer`,
//! `tfar_radio_effects_0_9`, `tfar_radio_effects_1_0`, `tfar_spatial_audio`,
//! `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** no global state; [`StereoBuffer`] owns its samples, every other function
//! works on caller buffers. Called from the TeamSpeak audio thread: only
//! [`StereoBuffer::with_frames`] and [`StereoBuffer::reserve_frames`] allocate by design, and a
//! [`StereoBuffer`] grows only when a block is larger than any before it.
//!
//! **Invariants:** gain and mixing saturate at the `i16` limits and never wrap; no adapter reads or
//! writes past `channels * frames` samples of a TeamSpeak block.

mod channel_views;
mod error;
mod float_conversion;
mod gain;
mod mixing;
pub mod prelude;
mod sample_saturation;
mod stereo_buffer;

pub use channel_views::{copy_to_stereo, mono_to_stereo, write_back_from_stereo};
pub use error::{Error, Result};
pub use float_conversion::{
    INPUT_SCALE, OUTPUT_SCALE, float_to_sample, from_float_planar, sample_to_float, to_float_planar,
};
pub use gain::{apply_gain, apply_stereo_gain};
pub use mixing::mix_into_additive;
pub use stereo_buffer::StereoBuffer;
