//! The four TeamSpeak audio callbacks: a speaker's voice, our own microphone, the final mix and
//! TeamSpeak's own 3D rolloff.
//!
//! **Role:** `onEditPostProcessVoiceDataEvent` (copy to stereo, the 0.9 voice chain through
//! `VoiceProcessor::process_sender`, write back), `onEditCapturedVoiceDataEvent` (our voice
//! through the chain for other players' loudspeakers, pushed into the mixer's `my_radio_voice`
//! stream through its non-blocking producer; the captured samples stay as they are),
//! `onEditMixedPlaybackVoiceDataEvent` (`PlaybackMixer::mix_into`) and `onCustom3dRolloffCalculationClientEvent` (gain 1, so
//! TeamSpeak's own 3D attenuation stays neutral), as
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:2600-2895` does them.
//!
//! **Position:** methods of `PluginCore`, called from the `Ts3Plugin` audio methods of
//! `TfarPlugin` on TeamSpeak's audio threads.
//!
//! **Signals and state:** reads the server map with `try_read`, passing the block through while
//! a connection is being added or removed; waits for a connection's state, as the original waits
//! on its critical section (`plugin.cpp:2659`), because a block passed through unprocessed
//! between processed ones is an audible jump (full level, no effect, no HRTF delay). Every wait
//! and every skipped block is counted in the core's `AudioDiagnostics`. The mixer takes its own
//! `try_lock`s.
//!
//! **Invariants:** no callback calls TeamSpeak or allocates for a block within the scratch's
//! size; the only wait is for a state, whose holders never call TeamSpeak or wait on anything; a
//! dormant plugin passes every block through untouched and leaves TeamSpeak's rolloff as it is.

use std::time::Instant;

use tfar_audio::sample_buffer::{copy_to_stereo, write_back_from_stereo};
use tfar_plugin_logic_0_9::voice_pipeline::{OwnVoiceContext, VoiceFrameContext, VoiceOutcome};
use tfar_ts3_host::{
    CapturedVoiceData, ClientId, MixedPlaybackVoiceData, PostProcessVoiceData,
    ServerConnectionHandlerId,
};

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
        let mut guard = self.lock_state_for_audio(connection);
        let ServerState {
            radio,
            voice,
            facts,
            voice_scratch,
            logged_voice_outcomes,
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
        log_voice_outcome_change(logged_voice_outcomes, radio, &context, outcome);
        if outcome != VoiceOutcome::PassThrough {
            // The counts were checked by the copy above, so the write back cannot fail.
            let _ = write_back_from_stereo(voice_scratch, data.samples, data.channels, data.frames);
        }
    }

    /// A block of our own microphone: in game and while it is sent, our voice through the chain
    /// as its own sender, pushed into the own-voice stream for loudspeakers of other players.
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
        let mut guard = self.lock_state_for_audio(connection);
        let ServerState {
            radio,
            voice,
            facts,
            captured_scratch,
            own_voice_stream,
            ..
        } = &mut *guard;
        let (Some(me), Some(own_voice_stream)) = (facts.my_client, own_voice_stream.as_mut())
        else {
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
            // The producer never blocks or allocates; past 500 ms it drops the oldest frames.
            own_voice_stream.push(own_voice.as_slice());
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

/// Logs a sender's voice outcome when it differs from the last one logged for that client id,
/// with the facts the voice chain decided on. Diagnostics for live sessions: one line per change,
/// so a player who stays processed or silenced costs nothing after the first frame.
fn log_voice_outcome_change(
    logged: &mut [u8],
    radio: &tfar_plugin_logic_0_9::client_state::ServerRadioData,
    context: &VoiceFrameContext<'_>,
    outcome: VoiceOutcome,
) {
    let code = match outcome {
        VoiceOutcome::Processed => 1,
        VoiceOutcome::Silenced => 2,
        VoiceOutcome::PassThrough => 3,
    };
    let Some(previous) = logged.get_mut(usize::from(context.sender.0)) else {
        return;
    };
    if *previous == code {
        return;
    }
    *previous = code;
    let fresh = radio.fresh_client_by_id(context.sender);
    tracing::info!(
        client = context.sender.0,
        outcome = ?outcome,
        nickname = fresh.map_or("<no fresh entry>", |(nickname, _)| nickname),
        has_fresh_data = radio.has_fresh_data(context.sender),
        listener_has_fresh_data = radio.has_fresh_data(context.listener),
        metadata_shows_plugin = context.sender_metadata_shows_plugin,
        channel = context.sender_channel_name,
        in_game = context.is_in_game,
        voice_volume = fresh.map(|(_, client)| client.voice_volume),
        "voice: outcome changed"
    );
}
