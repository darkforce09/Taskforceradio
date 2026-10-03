//! The four TeamSpeak audio callbacks: a speaker's voice, our own microphone, the final mix and
//! TeamSpeak's own 3D rolloff.
//!
//! **Role:** `onEditPostProcessVoiceDataEvent` (copy to stereo, the 0.9 voice chain through
//! `VoiceProcessor::process_sender`, write back), `onEditCapturedVoiceDataEvent` (our voice
//! through the chain for other players' loudspeakers, appended to the mixer's `my_radio_voice`
//! stream; the captured samples stay as they are), `onEditMixedPlaybackVoiceDataEvent`
//! (`PlaybackMixer::mix_into`) and `onCustom3dRolloffCalculationClientEvent` (gain 1, so
//! TeamSpeak's own 3D attenuation stays neutral), as
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:2600-2895` does them.
//!
//! **Position:** methods of `PluginCore`, called from the `Ts3Plugin` audio methods of
//! `TfarPlugin` on TeamSpeak's audio threads.
//!
//! **Signals and state:** reads the server map with `try_read` and a connection's state with
//! `try_lock`. When either is busy (a command thread is deciding, a connection is being added or
//! removed) the block passes through untouched for that frame, as the original does when it has
//! no data, instead of waiting; the mixer takes its own `try_lock`s.
//!
//! **Invariants:** no callback blocks, calls TeamSpeak or allocates for a block within the
//! scratch's size; a dormant plugin passes every block through untouched and leaves TeamSpeak's
//! rolloff as it is.

use std::time::Instant;

use tfar_sample_buffer::{copy_to_stereo, write_back_from_stereo};
use tfar_ts3_host::{
    CapturedVoiceData, ClientId, MixedPlaybackVoiceData, PostProcessVoiceData,
    ServerConnectionHandlerId,
};
use tfar_voice_pipeline_0_9::{OwnVoiceContext, VoiceFrameContext, VoiceOutcome, append_own_voice};

use crate::channel_snapshot::state_client_id;
use crate::plugin_core::PluginCore;
use crate::server_connection::ServerState;

/// The `edited` bit TeamSpeak sets while a captured block is going to be sent.
const CAPTURED_BLOCK_IS_SENT: i32 = 2;

impl PluginCore {
    /// A block of `data.client`'s voice: through the voice chain, or passed through.
    pub fn process_post_process_voice(&self, data: &mut PostProcessVoiceData<'_>) {
        if self.is_dormant() {
            return;
        }
        let Some(servers) = self.try_servers() else {
            return;
        };
        let Some(connection) = servers.get(&data.server) else {
            return;
        };
        let Some(mut guard) = connection.state.try_lock() else {
            return;
        };
        let ServerState {
            radio,
            voice,
            facts,
            voice_scratch,
            ..
        } = &mut *guard;
        let Some(listener) = facts.my_client else {
            return;
        };
        if copy_to_stereo(data.samples, data.channels, data.frames, voice_scratch).is_err() {
            return;
        }
        let sender = state_client_id(data.client);
        let context = VoiceFrameContext {
            sender,
            listener,
            sender_metadata_shows_plugin: facts.metadata_shows_plugin(sender),
            sender_channel_name: facts.channel_name_of(sender),
            is_in_game: self.is_in_game(),
        };
        let outcome = voice.process_sender(radio, &context, voice_scratch, Instant::now());
        if outcome != VoiceOutcome::PassThrough {
            // The counts were checked by the copy above, so the write back cannot fail.
            let _ = write_back_from_stereo(voice_scratch, data.samples, data.channels, data.frames);
        }
    }

    /// A block of our own microphone: in game and while it is sent, our voice through the chain
    /// as its own sender, appended to the own-voice stream for loudspeakers of other players.
    pub fn process_captured_voice(&self, data: &mut CapturedVoiceData<'_>) {
        if self.is_dormant() || !self.is_in_game() || *data.edited & CAPTURED_BLOCK_IS_SENT == 0 {
            return;
        }
        let Some(servers) = self.try_servers() else {
            return;
        };
        let Some(connection) = servers.get(&data.server) else {
            return;
        };
        let Some(mut guard) = connection.state.try_lock() else {
            return;
        };
        let ServerState {
            radio,
            voice,
            facts,
            captured_scratch,
            ..
        } = &mut *guard;
        let Some(me) = facts.my_client else {
            return;
        };
        if copy_to_stereo(data.samples, data.channels, data.frames, captured_scratch).is_err() {
            return;
        }
        let context = OwnVoiceContext {
            my_client_id: me,
            my_metadata_shows_plugin: facts.metadata_shows_plugin(me),
            my_channel_name: facts.channel_name_of(me),
            is_in_game: true,
        };
        if let Some(own_voice) =
            voice.process_own_voice_for_speakers(radio, &context, captured_scratch, Instant::now())
        {
            // A full stream table drops the block; the next one tries again.
            let _ = append_own_voice(&connection.mixer, own_voice);
        }
    }

    /// The final mix of `data.server`: every queued radio sound and our own voice added in.
    pub fn process_mixed_playback(&self, data: &mut MixedPlaybackVoiceData<'_>) {
        if self.is_dormant() {
            return;
        }
        let Some(servers) = self.try_servers() else {
            return;
        };
        if let Some(connection) = servers.get(&data.server) {
            // Wrong counts leave the block untouched, which is all a callback can do.
            let _ = connection.mixer.mix_into(
                data.samples,
                data.channels,
                data.frames,
                data.channel_fill_mask,
            );
        }
    }

    /// TeamSpeak's own 3D rolloff for `client`: full volume, so only the plugin's processing
    /// attenuates; a dormant plugin leaves TeamSpeak's value.
    pub fn neutral_rolloff(
        &self,
        _server: ServerConnectionHandlerId,
        _client: ClientId,
        volume: &mut f32,
    ) {
        if !self.is_dormant() {
            *volume = 1.0;
        }
    }
}
