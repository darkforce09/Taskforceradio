//! `PersonalRadioEffect`: the personal (short-wave) radio sound (`PersonalRadioEffect`,
//! `reference/tfar_1_0/ts/src/RadioEffect.hpp:206`).

#[cfg(test)]
#[path = "tests/personal.rs"]
mod tests;

use tfar_audio::dsp_filters::FilterDesign;

use crate::radio_effects::error::Result;
use crate::radio_effects::radio_effect::RadioEffect;

use super::speaker_chain::SpeakerChain;

/// The speaker high-pass: RBJ 900 Hz, Q 0.85.
const SPEAKER_HIGH_PASS: FilterDesign = FilterDesign::RbjHighPass {
    cutoff_hz: 900.0,
    quality: 0.85,
};

/// The speaker low-pass: RBJ 3000 Hz, Q 2.0.
const SPEAKER_LOW_PASS: FilterDesign = FilterDesign::RbjLowPass {
    cutoff_hz: 3000.0,
    quality: 2.0,
};

/// The personal radio effect: the speaker chain with a 900 Hz to 3000 Hz band and no microphone
/// filters before it.
#[derive(Debug, Clone)]
pub struct PersonalRadioEffect {
    speaker: SpeakerChain,
}

impl PersonalRadioEffect {
    /// A silent effect at error level 0. Allocates and designs its filters: call it off the
    /// audio thread.
    ///
    /// # Errors
    ///
    /// [`crate::radio_effects::Error::FilterDesign`] when a filter cannot be designed.
    pub fn new() -> Result<Self> {
        Ok(Self {
            speaker: SpeakerChain::new("personal", SPEAKER_HIGH_PASS, SPEAKER_LOW_PASS)?,
        })
    }
}

impl RadioEffect for PersonalRadioEffect {
    fn set_error_level(&mut self, distance_error: f64) {
        self.speaker.set_error_level(distance_error);
    }

    fn process(&mut self, block: &mut [f32]) {
        self.speaker.process(block);
    }
}
