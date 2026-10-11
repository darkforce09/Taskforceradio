//! Conversion between `i16` samples and the `f32` samples the filters and effects process.
//!
//! **Role:** turns stereo `i16` frames into two planar `f32` channels and back, with the scale
//! constants of the original plugins.
//!
//! **Position:** in `crates/audio`; called by the filter, dynamics and radio effect stages
//! around their `f32` processing. The specification is `processFilterStereo` in
//! `reference/tfar_1_0/ts/src/helpers.hpp` (input divided by `SHRT_MAX`) and `processRadioEffect` in
//! `reference/tfar_1_0/ts/src/RadioEffect.hpp` (output clamped to `[-1, 1]` times 32766).
//!
//! **Signals and state:** none; pure functions over caller buffers, called from the TeamSpeak audio
//! thread. Nothing allocates.
//!
//! **Invariants:** input is `sample / INPUT_SCALE`, so `i16::MAX` is exactly 1.0; output is
//! `trunc(clamp(value, -1, 1) * OUTPUT_SCALE)`, so it lies in `-32766..=32766` and `NaN` becomes 0;
//! a round trip moves a sample in `-32767..=32767` by at most 1, and `i16::MIN` comes back as
//! -32766.

use crate::sample_buffer::error::{Error, Result};
use crate::sample_buffer::sample_saturation::saturate_to_sample;
use crate::sample_buffer::stereo_buffer::StereoBuffer;

#[cfg(test)]
#[path = "tests/float_conversion.rs"]
mod tests;

/// The divisor from an `i16` sample to `f32`: `SHRT_MAX`, as the originals' filter input uses.
pub const INPUT_SCALE: f32 = 32767.0;

/// The multiplier from a clamped `f32` sample back to `i16`: `SHRT_MAX - 1`, as both originals'
/// filter and radio effect outputs use. It keeps a full-scale value one step inside the limit.
pub const OUTPUT_SCALE: f32 = 32766.0;

/// One `i16` sample as `f32`: `sample / INPUT_SCALE`, so `i16::MAX` is 1.0 and `i16::MIN` slightly
/// below -1.0.
pub fn sample_to_float(sample: i16) -> f32 {
    f32::from(sample) / INPUT_SCALE
}

/// One `f32` sample as `i16`: clamped to `[-1, 1]`, multiplied by [`OUTPUT_SCALE`] and truncated
/// toward zero; `NaN` becomes 0.
pub fn float_to_sample(value: f32) -> i16 {
    saturate_to_sample(value.clamp(-1.0, 1.0) * OUTPUT_SCALE)
}

/// Splits `stereo` into planar `f32` channels: frame `n` goes to `left[n]` and `right[n]` through
/// [`sample_to_float`]. Planar samples beyond `stereo.frames()` are untouched.
///
/// # Errors
///
/// [`Error::PlanarSliceTooShort`] when `left` or `right` holds fewer than `stereo.frames()`
/// samples; nothing is written then.
pub fn to_float_planar(stereo: &StereoBuffer, left: &mut [f32], right: &mut [f32]) -> Result<()> {
    check_planar_length("left", left.len(), stereo.frames())?;
    check_planar_length("right", right.len(), stereo.frames())?;
    for (([left_sample, right_sample], left_value), right_value) in stereo
        .frame_pairs()
        .iter()
        .zip(left.iter_mut())
        .zip(right.iter_mut())
    {
        *left_value = sample_to_float(*left_sample);
        *right_value = sample_to_float(*right_sample);
    }
    Ok(())
}

/// Joins planar `f32` channels back into `stereo`: `left[n]` and `right[n]` become frame `n`
/// through [`float_to_sample`], for every one of `stereo.frames()` frames. The frame count of
/// `stereo` stays; planar samples beyond it are ignored.
///
/// # Errors
///
/// [`Error::PlanarSliceTooShort`] when `left` or `right` holds fewer than `stereo.frames()`
/// samples; `stereo` is untouched then.
pub fn from_float_planar(left: &[f32], right: &[f32], stereo: &mut StereoBuffer) -> Result<()> {
    check_planar_length("left", left.len(), stereo.frames())?;
    check_planar_length("right", right.len(), stereo.frames())?;
    for (([left_sample, right_sample], left_value), right_value) in
        stereo.frame_pairs_mut().iter_mut().zip(left).zip(right)
    {
        *left_sample = float_to_sample(*left_value);
        *right_sample = float_to_sample(*right_value);
    }
    Ok(())
}

/// Refuses a planar slice of `actual` samples for a stereo buffer of `needed` frames.
fn check_planar_length(channel: &'static str, actual: usize, needed: usize) -> Result<()> {
    if actual < needed {
        return Err(Error::PlanarSliceTooShort {
            channel,
            needed,
            actual,
        });
    }
    Ok(())
}
