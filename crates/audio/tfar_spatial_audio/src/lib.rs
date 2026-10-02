//! The distance curve and emitter cone gains that replace X3DAudio, plus the simple stereo panner.
//!
//! **Role:** the gain model that replaces X3DAudio: the piecewise distance curve, the emitter cone
//! gain and the simple interaural level difference both versions use as their stereo fallback.
//!
//! **Position:** `crates/audio`, tier 1, depending on `tfar_geometry`, `tfar_sample_buffer`; used
//! by `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`. Direction itself comes from
//! `tfar_hrtf` (D20); this crate gives only level.
//!
//! **Signals and state:** none; pure gain functions and `Copy` value types, callable from the
//! TeamSpeak audio thread. Nothing allocates, construction included.
//!
//! **Invariants:** every gain is finite; curve gains are non-negative, cone volumes lie within 0
//! to 2 as X3DAudio requires, and the pan gains lie within `0.625 ± coefficient`.

mod crate_role;
mod distance_curve;
mod emitter_cone;
mod error;
mod interaural_level_difference;
pub mod prelude;

pub use crate_role::CRATE_ROLE;
pub use distance_curve::{CurvePoint, DistanceCurve};
pub use emitter_cone::EmitterCone;
pub use error::{Error, Result};
pub use interaural_level_difference::{
    ILD_BASE_GAIN, ILD_COEFFICIENT_0_9, ILD_COEFFICIENT_1_0, apply_pan, stereo_pan_gains,
};
