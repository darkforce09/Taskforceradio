//! The local player's own voice on its way to loudspeakers: the captured-voice path.
//!
//! **Role:** run the local player's captured voice through the voice chain as its own sender
//! (`ts3plugin_onEditCapturedVoiceDataEvent`, `reference/tfar_0_9_12/ts/src/plugin.cpp:2852`),
//! so that a loudspeaker radio of another player on the local player's transmit frequency
//! plays it, and hand the block to the playback mixer's own-voice stream.
//!
//! **Position:** methods of `VoiceProcessor` and a free function; the plugin calls them from
//! the captured-voice callback with the state locked, and appends the block to its
//! `PlaybackMixer`.
//!
//! **Signals and state:** uses the processor's own-voice scratch buffer; advances the local
//! player's sound state. TeamSpeak audio thread, inside the server lock.
//!
//! **Invariants:** nothing is produced outside a game, for a dead local player or without
//! current data for the local player; the direct voice of the local player is always silent,
//! so only loudspeaker receptions sound; the captured samples are never changed.

use std::time::Instant;

use tfar_client_state_0_9::ServerRadioData;
use tfar_playback_mixer::PlaybackMixer;
use tfar_sample_buffer::StereoBuffer;

use crate::error::{Error, Result};
use crate::voice_context::OwnVoiceContext;
use crate::voice_processor::VoiceProcessor;

#[cfg(test)]
#[path = "tests/own_voice.rs"]
mod tests;

/// The playback mixer stream the local player's own voice plays from (`appendPlayback`'s
/// `"my_radio_voice"`, plugin.cpp:2882).
pub const OWN_VOICE_STREAM_NAME: &str = "my_radio_voice";

impl VoiceProcessor {
    /// Runs `captured`, a stereo copy of the local player's microphone frame, through the voice
    /// chain with the local player as sender, and returns the block to append to the
    /// [`OWN_VOICE_STREAM_NAME`] stream; `None` outside a game, when the local player is dead or
    /// has no current data. `captured` itself is left as it is.
    pub fn process_own_voice_for_speakers(
        &mut self,
        state: &mut ServerRadioData,
        context: &OwnVoiceContext<'_>,
        captured: &StereoBuffer,
        now: Instant,
    ) -> Option<&StereoBuffer> {
        if !context.is_in_game || !state.is_alive || !state.has_fresh_data(context.my_client_id) {
            return None;
        }
        let mut own_voice = core::mem::take(&mut self.scratch.own_voice);
        own_voice.copy_from(captured);
        self.process_sender(state, &context.as_voice_frame(), &mut own_voice, now);
        self.scratch.own_voice = own_voice;
        Some(&self.scratch.own_voice)
    }
}

/// Appends `own_voice`, a block from [`VoiceProcessor::process_own_voice_for_speakers`], to the
/// [`OWN_VOICE_STREAM_NAME`] stream of `mixer`.
///
/// # Errors
///
/// [`Error::OwnVoiceStream`] when the mixer refuses the block (no stream slot is left).
pub fn append_own_voice(mixer: &PlaybackMixer, own_voice: &StereoBuffer) -> Result<()> {
    mixer
        .append_stream(OWN_VOICE_STREAM_NAME, own_voice.as_slice())
        .map_err(|source| Error::OwnVoiceStream { source })
}
