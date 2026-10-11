//! `SpeakerChain`: the chain the long-range and personal effects share (`SimpleRadioEffect`,
//! `reference/tfar_0_9_12/ts/src/RadioEffect.h:92`).
//!
//! **Role:** runs, per sample and in the original order, the 2399-sample delay line, the ring
//! modulation by the error level, the foldback distortion, the RBJ high-pass, the RBJ low-pass
//! and the ×30 make-up gain.
//!
//! **Position:** owned by `LongRangeRadioEffect` and `PersonalRadioEffect`, which give it their
//! speaker filters.
//!
//! **Signals and state:** the delay line, the ring-modulator phase, the mapped error level and
//! the two filters' memory. Built off the audio thread; processing runs on it and allocates
//! nothing.
//!
//! **Invariants:** the foldback threshold comes from the block as it enters the chain, before the
//! delay; the error level is always a value of `error_level_from_distance`; silence in gives
//! silence out.

use tfar_audio::dsp_filters::{FilterDesign, MonoFilter};

use crate::radio_effects::error::{Error, Result};
use crate::radio_effects::error_level::error_level_from_distance;
use crate::radio_effects::radio_effect::SAMPLE_RATE_HZ;
use crate::radio_effects::signal_stages::{
    DelayLine, RingModulator, foldback, foldback_threshold, narrow_to_f32,
};

/// The make-up gain after the speaker filters.
const MAKE_UP_GAIN: f32 = 30.0;

/// The delay, ring modulation, foldback and speaker filters of a 0.9.12 speaker effect.
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
            let delayed = self.delay_line.delay(*sample);
            let modulated = self.ring_modulator.modulate(delayed, self.error_level);
            let folded = foldback(modulated, threshold);
            let high_passed = narrow_to_f32(self.high_pass.process_sample(f64::from(folded)));
            let low_passed = self.low_pass.process_sample(f64::from(high_passed));
            *sample = narrow_to_f32(low_passed) * MAKE_UP_GAIN;
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
