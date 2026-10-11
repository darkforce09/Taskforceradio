//! The names a caller of `tfar_audio::spatial_audio` imports with `use crate::spatial_audio::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::spatial_audio::distance_curve::{CurvePoint, DistanceCurve};
pub use crate::spatial_audio::emitter_cone::EmitterCone;
pub use crate::spatial_audio::interaural_level_difference::{
    ILD_BASE_GAIN, ILD_COEFFICIENT_0_9, ILD_COEFFICIENT_1_0, apply_pan, stereo_pan_gains,
};
