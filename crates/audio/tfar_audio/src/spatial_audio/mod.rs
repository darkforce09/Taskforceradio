//! The distance curve and emitter cone gains that replace X3DAudio, plus the simple stereo panner.
//!
//! **Role:** the gain model that replaces X3DAudio: the piecewise distance curve, the emitter cone
//! gain and the simple interaural level difference both versions use as their stereo fallback.
//!
//! **Position:** a module of `tfar_audio`, depending on `tfar_foundation::geometry`, `tfar_audio::sample_buffer`; used
//! by `tfar_plugin_logic_0_9::voice_pipeline`, `tfar_plugin_logic_1_0::voice_pipeline`. Direction itself comes from
//! `tfar_audio::hrtf`; this module gives only level.
//!
//! **Signals and state:** none; pure gain functions and `Copy` value types, callable from the
//! TeamSpeak audio thread. Nothing allocates, construction included.
//!
//! **Invariants:** every gain is finite; curve gains are non-negative, cone volumes lie within 0
//! to 2 as X3DAudio requires, and the pan gains lie within `0.625 ± coefficient`.

mod distance_curve;
mod emitter_cone;
mod error;
mod interaural_level_difference;
pub mod prelude;

pub use distance_curve::{CurvePoint, DistanceCurve};
pub use emitter_cone::EmitterCone;
pub use error::{Error, Result};
pub use interaural_level_difference::{
    ILD_BASE_GAIN, ILD_COEFFICIENT_0_9, ILD_COEFFICIENT_1_0, apply_pan, stereo_pan_gains,
};
