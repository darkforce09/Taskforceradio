//! `AirborneRadioEffect`: the sound of a sender transmitting on an airborne-subtype radio, the
//! aircraft radios (`AirborneRadioEffect`, `reference/tfar_1_0/ts/src/RadioEffect.hpp:198`).

#[cfg(test)]
#[path = "tests/airborne.rs"]
mod tests;

use tfar_audio::dsp_filters::FilterDesign;

use crate::radio_effects::error::Result;
use crate::radio_effects::radio_effect::RadioEffect;

use super::speaker_chain::SpeakerChain;

/// The speaker high-pass: RBJ 1000 Hz, Q 1.0.
const SPEAKER_HIGH_PASS: FilterDesign = FilterDesign::RbjHighPass {
    cutoff_hz: 1000.0,
    quality: 1.0,
};

/// The speaker low-pass: RBJ 4000 Hz, Q 1.0.
const SPEAKER_LOW_PASS: FilterDesign = FilterDesign::RbjLowPass {
    cutoff_hz: 4000.0,
    quality: 1.0,
};

/// The airborne radio effect: the speaker chain with a 1000 Hz to 4000 Hz band.
#[derive(Debug, Clone)]
pub struct AirborneRadioEffect {
    speaker: SpeakerChain,
}

impl AirborneRadioEffect {
    /// A silent effect at error level 0. Allocates and designs its filters: call it off the
    /// audio thread.
    ///
    /// # Errors
    ///
    /// [`crate::radio_effects::Error::FilterDesign`] when a filter cannot be designed.
    pub fn new() -> Result<Self> {
        Ok(Self {
            speaker: SpeakerChain::new("airborne", SPEAKER_HIGH_PASS, SPEAKER_LOW_PASS)?,
        })
    }
}

impl RadioEffect for AirborneRadioEffect {
    fn set_error_level(&mut self, distance_error: f64) {
        self.speaker.set_error_level(distance_error);
    }

    fn process(&mut self, block: &mut [f32]) {
        self.speaker.process(block);
    }
}
