//! `UnderWaterRadioEffect`: the sound of a radio carried under water (`UnderWaterRadioEffect`,
//! `reference/tfar_0_9_12/ts/src/RadioEffect.h:56`).

#[cfg(test)]
#[path = "tests/under_water.rs"]
mod tests;

use tfar_dsp_filters::{FilterDesign, MonoFilter};

use crate::error::Result;
use crate::radio_effect::RadioEffect;
use crate::signal_stages::{SampleDropout, narrow_to_f32};

use super::speaker_chain::design;

/// The band-pass: Butterworth order 2 (four poles) centred on 1000 Hz, 400 Hz wide.
const BAND_PASS: FilterDesign = FilterDesign::ButterworthBandPass {
    order: 2,
    centre_hz: 1000.0,
    width_hz: 400.0,
};

/// The make-up gain after the band-pass.
const MAKE_UP_GAIN: f32 = 30.0;

/// The underwater radio effect: random sample dropouts with a probability of about the error
/// level, a Butterworth band-pass round 1000 Hz and ×30. The error level is the raw distance
/// error; no curve maps it.
#[derive(Debug, Clone)]
pub struct UnderWaterRadioEffect {
    dropout: SampleDropout,
    band_pass: MonoFilter,
}

impl UnderWaterRadioEffect {
    /// A silent effect at error level 0 whose dropouts draw from a generator seeded with
    /// `dropout_seed`; the same seed and input give the same output. Allocates and designs its
    /// filter: call it off the audio thread.
    ///
    /// # Errors
    ///
    /// [`crate::Error::FilterDesign`] when the band-pass cannot be designed.
    pub fn new(dropout_seed: u64) -> Result<Self> {
        Ok(Self {
            dropout: SampleDropout::new(dropout_seed),
            band_pass: design("underwater", "band-pass", BAND_PASS)?,
        })
    }
}

impl RadioEffect for UnderWaterRadioEffect {
    fn set_error_level(&mut self, distance_error: f64) {
        self.dropout.set_error_level(narrow_to_f32(distance_error));
    }

    fn process(&mut self, block: &mut [f32]) {
        for sample in block.iter_mut() {
            let kept = if self.dropout.should_drop() {
                0.0
            } else {
                *sample
            };
            *sample = narrow_to_f32(self.band_pass.process_sample(f64::from(kept))) * MAKE_UP_GAIN;
        }
    }
}
