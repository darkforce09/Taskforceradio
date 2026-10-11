//! `ClientDsp`: every sound object one sender's voice passes through.
//!
//! **Role:** own a sender's compressor, direct-voice state and per-radio state, make what a
//! `DspRequirements` names, and apply the original's two resets: the radio effect reset drops
//! objects, the voice reset only clears filter memory.
//!
//! **Position:** a field of `ClientData`; filled by `ClientData::ensure_dsp` on the command
//! threads, borrowed by the voice pipeline on the TeamSpeak audio thread.
//!
//! **Signals and state:** the compressor envelope, the filter and effect memories, the HRTF
//! convolution tails and the underwater dropout seed counter. Every object is made or dropped
//! while the plugin holds the server lock on a command thread; the audio thread only borrows.
//!
//! **Invariants:** `ensure` never replaces an object that exists (a vehicle low-pass is
//! redesigned only for a new isolation), so the audio thread keeps its filter memory from frame
//! to frame; the compressor is made once and never reset, as the original's; `reset_voices`
//! frees nothing; a radio entry that holds nothing after `reset_radio_effects` is removed.

use std::collections::BTreeMap;

use tfar_audio::dynamics::{Compressor, CompressorSettings};
use tfar_audio::hrtf::HrirLibrary;

use super::direct_voice_dsp::DirectVoiceDsp;
use super::dsp_requirements::DspRequirements;
use super::radio_dsp::RadioDsp;
use crate::client_state::error::Result;

#[cfg(test)]
#[path = "tests/client_dsp.rs"]
mod tests;

/// The seed of a client's first underwater effect; each later one takes the next value of
/// [`next_seed`].
const FIRST_DROPOUT_SEED: u64 = 0x5446_4152_0009_0012;

/// One sender's sound state.
#[derive(Debug)]
pub struct ClientDsp {
    compressor: Option<Compressor>,
    direct_voice: DirectVoiceDsp,
    radios: BTreeMap<String, RadioDsp>,
    next_dropout_seed: u64,
}

impl Default for ClientDsp {
    fn default() -> Self {
        Self {
            compressor: None,
            direct_voice: DirectVoiceDsp::default(),
            radios: BTreeMap::new(),
            next_dropout_seed: FIRST_DROPOUT_SEED,
        }
    }
}

impl ClientDsp {
    /// The compressor the radio path runs once per frame (`VERSION_0_9` settings), when made.
    pub fn compressor_mut(&mut self) -> Option<&mut Compressor> {
        self.compressor.as_mut()
    }

    /// The direct voice's state.
    pub fn direct_voice_mut(&mut self) -> &mut DirectVoiceDsp {
        &mut self.direct_voice
    }

    /// The state of the radio `radio_id`, when made. Looking it up allocates nothing.
    pub fn radio_mut(&mut self, radio_id: &str) -> Option<&mut RadioDsp> {
        self.radios.get_mut(radio_id)
    }

    /// The radio ids that hold state, in order.
    pub fn radio_ids(&self) -> impl Iterator<Item = &str> {
        self.radios.keys().map(String::as_str)
    }

    /// Whether the sender holds no sound object but, perhaps, its compressor.
    pub fn holds_no_voice_state(&self) -> bool {
        self.direct_voice.is_empty() && self.radios.is_empty()
    }

    /// Makes every object `requirements` names that does not exist yet, and the compressor;
    /// direction sources only when `hrtf` is given. Allocates: call it off the audio thread.
    ///
    /// # Errors
    ///
    /// [`crate::client_state::Error`] when a filter, effect or the compressor cannot be built; the objects made
    /// before the failure stay.
    pub fn ensure(
        &mut self,
        requirements: &DspRequirements,
        hrtf: Option<&HrirLibrary>,
    ) -> Result<()> {
        if self.compressor.is_none() {
            self.compressor = Some(Compressor::new(CompressorSettings::VERSION_0_9)?);
        }
        if let Some(direct_voice) = &requirements.direct_voice {
            self.direct_voice.ensure(direct_voice, hrtf)?;
        }
        for requirement in &requirements.radios {
            let seed = self.next_dropout_seed;
            self.next_dropout_seed = next_seed(seed);
            self.radios
                .entry(requirement.radio_id.clone())
                .or_default()
                .ensure(requirement, hrtf, seed)?;
        }
        Ok(())
    }

    /// Drops every radio effect and the loudspeaker and phone filters (`resetRadioEffect`).
    pub fn reset_radio_effects(&mut self) {
        for radio in self.radios.values_mut() {
            radio.reset_radio_effects();
        }
        self.radios.retain(|_, radio| !radio.is_empty());
    }

    /// Clears the memory of the can't-speak, under-water and vehicle filters (`resetVoices`),
    /// keeping every object: in serious mode this runs on every `POS` of a silent player, so it
    /// must neither free nor rebuild a two-megabyte direction source under the server lock the
    /// audio thread waits for. The direction sources restart themselves after an idle gap.
    pub fn reset_voices(&mut self) {
        self.direct_voice.reset();
        for radio in self.radios.values_mut() {
            radio.reset_voices();
        }
    }
}

/// The seed after `seed`: one step of the `SplitMix64` generator, so successive effects drop different samples.
fn next_seed(seed: u64) -> u64 {
    let mut mixed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}
