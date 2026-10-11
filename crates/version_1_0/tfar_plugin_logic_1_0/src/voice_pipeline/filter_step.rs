//! The shared step of a filter that may not exist yet: filter at a gain, or the gain alone.

use tfar_audio::dsp_filters::StereoFilter;
use tfar_audio::sample_buffer::{StereoBuffer, apply_gain};

/// Runs `filter` over `samples` at `gain`, as `processFilterStereo` does; when the filter was
/// never ensured, applies only `gain`, so the level stays right while the muffling is missing.
pub(crate) fn filter_or_gain(
    filter: Option<&mut StereoFilter>,
    samples: &mut StereoBuffer,
    gain: f32,
) {
    match filter {
        Some(filter) => filter.process_stereo_i16(samples, gain),
        None => apply_gain(samples.as_mut_slice(), gain),
    }
}
