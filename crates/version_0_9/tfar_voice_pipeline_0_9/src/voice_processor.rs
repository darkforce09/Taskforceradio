//! `VoiceProcessor`: the 0.9.12 stereo function for one sender's post-process voice frame.
//!
//! **Role:** decide whether a voice frame passes through, goes silent or runs the voice chain
//! (the gates of `ts3plugin_onEditPostProcessVoiceDataEventStereo`,
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:2653-2828`), and run the chain with the scratch it
//! owns.
//!
//! **Position:** the crate's entry point; the plugin holds one per server connection beside the
//! `ServerRadioData` and calls it from TeamSpeak's post-process and captured-voice callbacks
//! with the state already locked; it calls `voice_chain/`.
//!
//! **Signals and state:** owns the [`VoiceScratch`] and a count of frames that met missing sound
//! state; refreshes the sender's plugin-enabled cache; advances the sender's sound state. Runs on
//! the TeamSpeak audio thread inside the server lock and makes no TeamSpeak call.
//!
//! **Invariants:** a frame passes through untouched only when the original leaves it alone; a
//! dead listener hears silence from a sender in the serious channel; the local player's own
//! frame without current data is silence; the plugin-enabled cache is consulted only where the
//! original's short-circuit consults it; the processor makes and drops no sound object.

use std::time::Instant;

use tfar_client_state_0_9::ServerRadioData;
use tfar_radio_routing_0_9::{is_other_radio_plugin_channel, is_serious_mode};
use tfar_sample_buffer::StereoBuffer;

use crate::voice_chain::{plan_sender, render_frame};
use crate::voice_context::{VoiceFrameContext, VoiceOutcome};
use crate::voice_scratch::VoiceScratch;

#[cfg(test)]
#[path = "tests/voice_processor.rs"]
mod tests;

/// The voice chain of one server connection, with the scratch it works in.
#[derive(Debug, Default)]
pub struct VoiceProcessor {
    pub(crate) scratch: VoiceScratch,
    frames_missing_sound_state: u64,
}

impl VoiceProcessor {
    /// A processor whose scratch holds frames of up to `max_frames` frames without growing.
    /// Allocates: call it off the audio thread, once per server connection.
    pub fn new(max_frames: usize) -> Self {
        Self {
            scratch: VoiceScratch::with_frames(max_frames),
            frames_missing_sound_state: 0,
        }
    }

    /// The scratch buffers, to check their capacities.
    pub fn scratch(&self) -> &VoiceScratch {
        &self.scratch
    }

    /// How many processed frames had a path left silent because a sound object it needed was
    /// not made yet (the plugin makes them with `ClientData::ensure_dsp` while handling
    /// commands).
    pub fn frames_missing_sound_state(&self) -> u64 {
        self.frames_missing_sound_state
    }

    /// Processes `samples`, one post-process frame of the sender `context.sender`, in place, with
    /// `state` locked by the caller; `now` dates the plugin-enabled check.
    ///
    /// With current data for sender and listener and the plugin enabled for the sender, the
    /// frame is silenced when the sender sits in the serious channel and the listener is dead,
    /// and runs the voice chain otherwise. Without, the local player's own frame is silenced, an
    /// alive in-game listener hears a serious-channel TFAR player without current data as
    /// silence, and everyone else passes through untouched.
    pub fn process_sender(
        &mut self,
        state: &mut ServerRadioData,
        context: &VoiceFrameContext<'_>,
        samples: &mut StereoBuffer,
        now: Instant,
    ) -> VoiceOutcome {
        let is_serious = is_serious_mode(context.sender_channel_name, &state.serious_channel_name);
        let has_current_data =
            state.has_fresh_data(context.sender) && state.has_fresh_data(context.listener);
        if has_current_data && self.is_plugin_enabled_for_sender(state, context, now) {
            if is_serious && !state.is_alive {
                samples.clear();
                return VoiceOutcome::Silenced;
            }
            return self.run_chain(state, context, samples);
        }
        if context.sender == context.listener {
            samples.clear();
            return VoiceOutcome::Silenced;
        }
        if !is_other_radio_plugin_channel(context.sender_channel_name)
            && is_serious
            && state.is_alive
            && context.is_in_game
            && self.is_plugin_enabled_for_sender(state, context, now)
        {
            samples.clear();
            return VoiceOutcome::Silenced;
        }
        VoiceOutcome::PassThrough
    }

    /// Plans and renders the frame of a sender with current data.
    fn run_chain(
        &mut self,
        state: &mut ServerRadioData,
        context: &VoiceFrameContext<'_>,
        samples: &mut StereoBuffer,
    ) -> VoiceOutcome {
        let scratch = &mut self.scratch;
        let plan = {
            let server: &ServerRadioData = state;
            let (Some((nickname, sender)), Some((_, listener))) = (
                server.fresh_client_by_id(context.sender),
                server.fresh_client_by_id(context.listener),
            ) else {
                return VoiceOutcome::PassThrough;
            };
            scratch.sender_nickname.clear();
            scratch.sender_nickname.push_str(nickname);
            plan_sender(
                server,
                nickname,
                sender,
                listener,
                &mut scratch.heard,
                &mut scratch.routes,
            )
        };
        let Some(sender) = state.clients.get_mut(scratch.sender_nickname.as_str()) else {
            scratch.release_routes();
            return VoiceOutcome::PassThrough;
        };
        let is_complete = render_frame(&plan, &mut sender.dsp, samples, scratch);
        scratch.release_routes();
        if !is_complete {
            self.frames_missing_sound_state = self.frames_missing_sound_state.saturating_add(1);
        }
        VoiceOutcome::Processed
    }

    /// `isPluginEnabledForUser` (`reference/tfar_0_9_12/ts/src/plugin.cpp:2526`): the
    /// metadata check, kept up by the sender's plugin-enabled cache. The cache is the sender's
    /// current entry, else any entry with the sender's client id; without one the metadata
    /// decides alone.
    fn is_plugin_enabled_for_sender(
        &mut self,
        state: &mut ServerRadioData,
        context: &VoiceFrameContext<'_>,
        now: Instant,
    ) -> bool {
        let metadata_shows_plugin = context.sender_metadata_shows_plugin;
        let nickname = &mut self.scratch.sender_nickname;
        let has_current_entry = state
            .fresh_nickname_of(context.sender)
            .map(|fresh| {
                nickname.clear();
                nickname.push_str(fresh);
            })
            .is_some();
        let entry = if has_current_entry {
            state.clients.get_mut(nickname.as_str())
        } else {
            state
                .clients
                .values_mut()
                .find(|client| client.client_id == context.sender)
        };
        entry.map_or(metadata_shows_plugin, |client| {
            client.refresh_plugin_enabled(metadata_shows_plugin, now)
        })
    }
}
