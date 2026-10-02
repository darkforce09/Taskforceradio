//! `SpeakerChain`: the chain the long-range, airborne and personal effects share
//! (`SimpleRadioEffect`, `reference/tfar_1_0/ts/src/RadioEffect.hpp:86`).
//!
//! **Role:** runs, per sample and in the original order, the foldback distortion, the ×30 gain,
//! the 2399-sample delay line and the ring modulation by the error level, then the RBJ high-pass
//! and the RBJ low-pass.
//!
//! **Position:** owned by `LongRangeRadioEffect`, `AirborneRadioEffect` and
//! `PersonalRadioEffect`, which give it their speaker filters.
//!
//! **Signals and state:** the delay line, the ring-modulator phase, the mapped error level and
//! the two filters' memory. Built off the audio thread; processing runs on it and allocates
//! nothing.
//!
//! **Invariants:** the foldback threshold comes from the block as it enters the chain; the error
//! level is always a value of `error_level_from_distance`; silence in gives silence out.

use tfar_dsp_filters::{FilterDesign, MonoFilter};

use crate::error::{Error, Result};
use crate::error_level::error_level_from_distance;
use crate::radio_effect::SAMPLE_RATE_HZ;
use crate::signal_stages::{DelayLine, RingModulator, foldback, foldback_threshold, narrow_to_f32};

/// The gain between the foldback and the delay line.
const PRE_DELAY_GAIN: f32 = 30.0;

/// The foldback, gain, delay, ring modulation and speaker filters of a 1.0 speaker effect.
#[derive(Debug, Clone)]
pub(crate) struct SpeakerChain {
    delay_line: DelayLine,
    ring_modulator: RingModulator,
    error_level: f32,
    high_pass: MonoFilter,
    low_pass: MonoFilter,
}

impl SpeakerChain {
    /// A silent chain at error level 0 with the speaker filters `high_pass` and `low_pass`.
    /// Allocates and designs: call it off the audio thread.
    ///
    /// # Errors
    ///
    /// [`Error::FilterDesign`] when a filter cannot be designed.
    pub(crate) fn new(
        effect: &'static str,
        high_pass: FilterDesign,
        low_pass: FilterDesign,
    ) -> Result<Self> {
        Ok(Self {
            delay_line: DelayLine::new(),
            ring_modulator: RingModulator::default(),
            error_level: 0.0,
            high_pass: design(effect, "speaker high-pass", high_pass)?,
            low_pass: design(effect, "speaker low-pass", low_pass)?,
        })
    }

    /// Sets the error level to `error_level_from_distance(distance_error)`.
    pub(crate) fn set_error_level(&mut self, distance_error: f64) {
        self.error_level = narrow_to_f32(error_level_from_distance(distance_error));
    }

    /// Runs the chain over `block` in place.
    pub(crate) fn process(&mut self, block: &mut [f32]) {
        if block.is_empty() {
            return;
        }
        let threshold = foldback_threshold(block, self.error_level);
        for sample in block.iter_mut() {
            let folded = foldback(*sample, threshold) * PRE_DELAY_GAIN;
            let delayed = self.delay_line.delay(folded);
            let modulated = self.ring_modulator.modulate(delayed, self.error_level);
            let high_passed = narrow_to_f32(self.high_pass.process_sample(f64::from(modulated)));
            *sample = narrow_to_f32(self.low_pass.process_sample(f64::from(high_passed)));
        }
    }
}

/// Designs one filter of `effect` at [`SAMPLE_RATE_HZ`], naming it in the error.
///
/// # Errors
///
/// [`Error::FilterDesign`] when the filter library refuses the design.
pub(crate) fn design(
    effect: &'static str,
    filter: &'static str,
    filter_design: FilterDesign,
) -> Result<MonoFilter> {
    MonoFilter::new(filter_design, SAMPLE_RATE_HZ).map_err(|source| Error::FilterDesign {
        effect,
        filter,
        source,
    })
}
