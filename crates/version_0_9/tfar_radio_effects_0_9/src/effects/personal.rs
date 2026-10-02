//! `PersonalRadioEffect`: the personal (short-wave) radio sound (`PersonalRadioEffect`,
//! `reference/tfar_0_9_12/ts/src/RadioEffect.h:194`).
//!
//! **Role:** runs the microphone band (RBJ high-pass 900 Hz, low-pass 3000 Hz) over the whole
//! block, then the speaker chain with its 520 Hz to 4300 Hz band.
//!
//! **Position:** created per sender and SW radio by `tfar_client_state_0_9`, run by
//! `tfar_voice_pipeline_0_9` through `process_radio_effect`.
//!
//! **Signals and state:** the two microphone filters' memory and the speaker chain's state. Built
//! off the audio thread; processing runs on it and allocates nothing.
//!
//! **Invariants:** the microphone filters run before the block's level is measured, so the
//! foldback threshold follows the microphone-filtered signal.

#[cfg(test)]
#[path = "tests/personal.rs"]
mod tests;

use tfar_dsp_filters::{FilterDesign, MonoFilter};

use crate::error::Result;
use crate::radio_effect::RadioEffect;
use crate::signal_stages::narrow_to_f32;

use super::speaker_chain::{SpeakerChain, design};

/// The microphone high-pass that runs before the speaker chain: RBJ 900 Hz, Q 0.85.
const MICROPHONE_HIGH_PASS: FilterDesign = FilterDesign::RbjHighPass {
    cutoff_hz: 900.0,
    quality: 0.85,
};

/// The microphone low-pass that runs before the speaker chain: RBJ 3000 Hz, Q 2.0.
const MICROPHONE_LOW_PASS: FilterDesign = FilterDesign::RbjLowPass {
    cutoff_hz: 3000.0,
    quality: 2.0,
};

/// The speaker high-pass: RBJ 520 Hz, Q 0.97.
const SPEAKER_HIGH_PASS: FilterDesign = FilterDesign::RbjHighPass {
    cutoff_hz: 520.0,
    quality: 0.97,
};

/// The speaker low-pass: RBJ 4300 Hz, Q 2.0.
const SPEAKER_LOW_PASS: FilterDesign = FilterDesign::RbjLowPass {
    cutoff_hz: 4300.0,
    quality: 2.0,
};

/// The personal radio effect: the microphone band 900 Hz to 3000 Hz over the whole block, then
/// the speaker chain with a 520 Hz to 4300 Hz band, so the level that sets the foldback threshold
/// is the microphone-filtered one.
#[derive(Debug, Clone)]
pub struct PersonalRadioEffect {
    microphone_high_pass: MonoFilter,
    microphone_low_pass: MonoFilter,
    speaker: SpeakerChain,
}

impl PersonalRadioEffect {
    /// A silent effect at error level 0. Allocates and designs its filters: call it off the
    /// audio thread.
    ///
    /// # Errors
    ///
    /// [`crate::Error::FilterDesign`] when a filter cannot be designed.
    pub fn new() -> Result<Self> {
        Ok(Self {
            microphone_high_pass: design("personal", "microphone high-pass", MICROPHONE_HIGH_PASS)?,
            microphone_low_pass: design("personal", "microphone low-pass", MICROPHONE_LOW_PASS)?,
            speaker: SpeakerChain::new("personal", SPEAKER_HIGH_PASS, SPEAKER_LOW_PASS)?,
        })
    }
}

impl RadioEffect for PersonalRadioEffect {
    fn set_error_level(&mut self, distance_error: f64) {
        self.speaker.set_error_level(distance_error);
    }

    fn process(&mut self, block: &mut [f32]) {
        for sample in block.iter_mut() {
            let high_passed =
                narrow_to_f32(self.microphone_high_pass.process_sample(f64::from(*sample)));
            *sample = narrow_to_f32(
                self.microphone_low_pass
                    .process_sample(f64::from(high_passed)),
            );
        }
        self.speaker.process(block);
    }
}
