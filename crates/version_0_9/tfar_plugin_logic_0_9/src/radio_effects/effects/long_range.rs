//! `LongRangeRadioEffect`: the long-range (backpack and vehicle) radio sound, which 0.9.12 also
//! plays for airborne radios (`LongRangeRadioffect`,
//! `reference/tfar_0_9_12/ts/src/RadioEffect.h:183`).

#[cfg(test)]
#[path = "tests/long_range.rs"]
mod tests;

use tfar_audio::dsp_filters::FilterDesign;

use crate::radio_effects::error::Result;
use crate::radio_effects::radio_effect::RadioEffect;

use super::speaker_chain::SpeakerChain;

/// The speaker high-pass: RBJ 520 Hz, Q 0.97.
const SPEAKER_HIGH_PASS: FilterDesign = FilterDesign::RbjHighPass {
    cutoff_hz: 520.0,
    quality: 0.97,
};

/// The speaker low-pass: RBJ 1300 Hz, Q 1.0.
const SPEAKER_LOW_PASS: FilterDesign = FilterDesign::RbjLowPass {
    cutoff_hz: 1300.0,
    quality: 1.0,
};

/// The long-range radio effect: the speaker chain with a 520 Hz to 1300 Hz band.
#[derive(Debug, Clone)]
pub struct LongRangeRadioEffect {
    speaker: SpeakerChain,
}

impl LongRangeRadioEffect {
    /// A silent effect at error level 0. Allocates and designs its filters: call it off the
    /// audio thread.
    ///
    /// # Errors
    ///
    /// [`crate::radio_effects::Error::FilterDesign`] when a filter cannot be designed.
    pub fn new() -> Result<Self> {
        Ok(Self {
            speaker: SpeakerChain::new("long-range", SPEAKER_HIGH_PASS, SPEAKER_LOW_PASS)?,
        })
    }
}

impl RadioEffect for LongRangeRadioEffect {
    fn set_error_level(&mut self, distance_error: f64) {
        self.speaker.set_error_level(distance_error);
    }

    fn process(&mut self, block: &mut [f32]) {
        self.speaker.process(block);
    }
}
