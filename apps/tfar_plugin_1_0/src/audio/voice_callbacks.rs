//! The four audio callbacks: a speaker's voice through the 1.0 chain, the own microphone for
//! speaker radios, the mixer into the final playback, and TeamSpeak's own 3D rolloff kept neutral.
//!
//! **Role:** `onEditPostProcessVoiceDataEvent`, `onEditCapturedVoiceDataEvent`,
//! `onEditMixedPlaybackVoiceDataEvent` and `onCustom3dRolloffCalculationClientEvent`
//! (`reference/tfar_1_0/ts/src/plugin.cpp:747-823`, `PlaybackHandler.cpp:36-70`).
//!
//! **Position:** called by `TfarPlugin` on TeamSpeak's audio threads with the runtime loaded
//! from its `ArcSwap`.
//!
//! **Signals and state:** reads the settings snapshot and the server state; the server registry
//! and the server lock are taken with `try_read` and `try_lock` only.
//!
//! **Invariants:** no callback blocks, calls TeamSpeak or allocates once the scratch fits the
//! block: a busy lock, a dormant plugin, no game or an unknown sender passes the block through
//! untouched, as the original does when its data is missing; metadata comes from the cache the
//! event handlers and the service thread fill.

use std::time::Instant;

use tfar_sample_buffer::{copy_to_stereo, write_back_from_stereo};
use tfar_ts3_host::{CapturedVoiceData, MixedPlaybackVoiceData, PostProcessVoiceData};
use tfar_voice_pipeline_1_0::prelude::{VoiceFrameContext, VoiceOutcome};

use crate::runtime::PluginRuntime;
use crate::runtime::server_state::{ServerState, state_client_id};

/// The `edited` bit TeamSpeak sets while the captured block is going to be sent.
const CAPTURE_IS_SENT: i32 = 2;

/// A speaker's voice block: copied to stereo, through `process_sender`, written back; passed
/// through untouched when the chain leaves it as it is or the state is busy.
pub fn post_process_voice(runtime: &PluginRuntime, data: &mut PostProcessVoiceData<'_>) {
    profiling::scope!("post_process_voice");
    if runtime.is_dormant() || !runtime.is_in_game() {
        return;
    }
    let Some(shared) = runtime.servers.try_get(data.server) else {
        return;
    };
    // Busy means the game link or an event writes the state: pass the block through.
    let Some(mut state) = shared.try_lock() else {
        return;
    };
    let settings = runtime.settings_snapshot();
    let now = Instant::now();
    let sender = state_client_id(data.client);
    let ServerState {
        session,
        voice,
        playback_stereo,
        client_metadata,
        antennas,
        is_serious_mode,
        ..
    } = &mut *state;
    if copy_to_stereo(data.samples, data.channels, data.frames, playback_stereo).is_err() {
        return;
    }
    let sender_metadata = session
        .needs_plugin_metadata(sender, now)
        .then(|| client_metadata.get(&sender).map(String::as_str))
        .flatten();
    let context = VoiceFrameContext {
        settings: &settings,
        antennas,
        is_in_game: true,
        is_serious_mode: *is_serious_mode,
        sender_metadata,
        now,
    };
    match voice.process_sender(session, &context, sender, playback_stereo) {
        VoiceOutcome::PassThrough => {}
        VoiceOutcome::Processed | VoiceOutcome::Silenced => {
            // The counts were checked by the copy in, so the write back cannot fail.
            let _ = write_back_from_stereo(
                playback_stereo,
                &mut *data.samples,
                data.channels,
                data.frames,
            );
        }
    }
}

/// The own microphone, in game and while sent: through the chain for the speaker radios that
/// play it, appended to the `my_radio_voice` stream; the outgoing samples stay unchanged.
pub fn captured_voice(runtime: &PluginRuntime, data: &mut CapturedVoiceData<'_>) {
    profiling::scope!("captured_voice");
    if runtime.is_dormant() || !runtime.is_in_game() || *data.edited & CAPTURE_IS_SENT == 0 {
        return;
    }
    let Some(shared) = runtime.servers.try_get(data.server) else {
        return;
    };
    let Some(mut state) = shared.try_lock() else {
        return;
    };
    let settings = runtime.settings_snapshot();
    let now = Instant::now();
    let ServerState {
        session,
        voice,
        captured_stereo,
        client_metadata,
        antennas,
        is_serious_mode,
        ..
    } = &mut *state;
    if session.game.speakers.is_empty() {
        return;
    }
    let Some(own) = session.clients.own_client_id() else {
        return;
    };
    if copy_to_stereo(data.samples, data.channels, data.frames, captured_stereo).is_err() {
        return;
    }
    let sender_metadata = session
        .needs_plugin_metadata(own, now)
        .then(|| client_metadata.get(&own).map(String::as_str))
        .flatten();
    let context = VoiceFrameContext {
        settings: &settings,
        antennas,
        is_in_game: true,
        is_serious_mode: *is_serious_mode,
        sender_metadata,
        now,
    };
    if let Some(voice) = voice.process_own_voice_for_speakers(session, &context, captured_stereo)
        && !runtime.sounds.push_own_voice(voice.as_slice())
    {
        tracing::trace!("the own voice did not reach the speaker stream");
    }
}

/// The final mix of the server in front: every queued radio sound and the own-voice stream are
/// added into channels 0 and 1, saturating.
pub fn mixed_playback(runtime: &PluginRuntime, data: &mut MixedPlaybackVoiceData<'_>) {
    profiling::scope!("mixed_playback");
    if runtime.is_dormant() || data.server != runtime.front_server() {
        return;
    }
    if let Err(error) = runtime.sounds.mixer.mix_into(
        data.samples,
        data.channels,
        data.frames,
        data.channel_fill_mask,
    ) {
        tracing::trace!(%error, "the mixed playback block was refused");
    }
}

/// TeamSpeak's own 3D attenuation stays at 1, so only TFAR's distance rules apply.
pub fn custom_3d_rolloff(runtime: &PluginRuntime, volume: &mut f32) {
    if !runtime.is_dormant() {
        *volume = 1.0;
    }
}
