//! The per-sample stages the effects chain together: the delay line, the ring modulator, the
//! foldback distortion, the sample dropouts and the `f64` to `f32` narrowing after each filter.

mod delay_line;
mod float_narrowing;
mod foldback;
mod ring_modulator;
mod sample_dropout;

pub(crate) use delay_line::DelayLine;
pub(crate) use float_narrowing::narrow_to_f32;
pub(crate) use foldback::{foldback, foldback_threshold};
pub(crate) use ring_modulator::RingModulator;
pub(crate) use sample_dropout::SampleDropout;
