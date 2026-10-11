//! The names a caller of `tfar_sample_buffer` imports with `use tfar_sample_buffer::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::channel_views::{copy_to_stereo, mono_to_stereo, write_back_from_stereo};
pub use crate::float_conversion::{
    INPUT_SCALE, OUTPUT_SCALE, float_to_sample, from_float_planar, sample_to_float, to_float_planar,
};
pub use crate::gain::{apply_gain, apply_stereo_gain};
pub use crate::mixing::mix_into_additive;
pub use crate::stereo_buffer::StereoBuffer;
