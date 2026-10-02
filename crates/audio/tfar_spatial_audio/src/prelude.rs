//! The names a caller of `tfar_spatial_audio` imports with `use tfar_spatial_audio::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
pub use crate::distance_curve::{CurvePoint, DistanceCurve};
pub use crate::emitter_cone::EmitterCone;
pub use crate::interaural_level_difference::{
    ILD_BASE_GAIN, ILD_COEFFICIENT_0_9, ILD_COEFFICIENT_1_0, apply_pan, stereo_pan_gains,
};
