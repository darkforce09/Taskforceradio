//! Plugin commands from other TFAR clients: remote tangents, `VOLUME`, `RELEASE_ALL_TANGENTS`
//! and `REQVOL`, and the own `VOLUME` broadcast when the local client starts or stops talking.
//!
//! **Role:** `processPluginCommand`, `processTangentPress` and
//! `onClientSelfVariableUpdateEvent` (`reference/tfar_1_0/ts/src/plugin.cpp:829-1086`) over the
//! codec of `tfar_plugin_commands_1_0` and the apply functions of `tfar_client_state_1_0` and
//! `tfar_radio_routing_1_0`.
//!
//! **Position:** called by `TfarPlugin` on TeamSpeak's main thread.
//!
//! **Signals and state:** the server state under its lock; the remote radio sounds are shaped and
//! queued after it is released.
//!
//! **Invariants:** only commands of the server in front from a plugin whose name starts with
//! `TFAR` count; a sender starting a transmission gets fresh direction sources.

use std::time::Instant;

use tfar_client_state_1_0::prelude::RemoteRadioSounds;
use tfar_plugin_commands_1_0::prelude::{PluginCommand, accepts_sender, decode, encode};
use tfar_radio_routing_1_0::apply_remote_tangent;
use tfar_ts3_host::{ClientProperty, PluginCommandTarget, ServerConnectionHandlerId};
use tfar_voice_pipeline_1_0::prelude::{RemoteSoundListener, RemoteSoundPlan};

use crate::runtime::PluginRuntime;
use crate::runtime::mute_updates::decide_client_mute;
use crate::runtime::server_state::host_client_id;

/// A plugin command arrived on `server` from the plugin `plugin_name`.
pub fn plugin_command_received(
    runtime: &PluginRuntime,
    server: ServerConnectionHandlerId,
    plugin_name: &str,
    command: &str,
) {
    if server != runtime.ts().current_server() || !accepts_sender(plugin_name) {
        tracing::debug!(plugin_name, "a plugin command from elsewhere is ignored");
        return;
    }
    let command = match decode(command) {
        Ok(command) => command,
        Err(error) => {
            tracing::debug!(%error, "an unknown plugin command is ignored");
            return;
        }
    };
    let Some(state) = runtime.servers.get(server) else {
        return;
    };
    let settings = runtime.settings();
    let now = Instant::now();
    match command {
        PluginCommand::Tangent(tangent) => {
            let (effects, mute, listener, speaker_distance) = {
                let mut state = state.lock();
                let crate::runtime::ServerState {
                    session, antennas, ..
                } = &mut *state;
                let Some(effects) =
                    apply_remote_tangent(session, &settings, antennas, &tangent, now)
                else {
                    return;
                };
                let heard = Some(!effects.heard.is_empty());
                let mute = decide_client_mute(&mut state, &settings, effects.client_id, heard, now);
                runtime.ensure_sender_dsp(&mut state, &settings, effects.client_id);
                let listener = state.session.clients.own().map(|own| RemoteSoundListener {
                    position: own.client_position(now),
                    view: own.view_direction,
                });
                (effects, mute, listener, state.session.game.speaker_distance)
            };
            // A dormant plugin mixes nothing, so its sounds would only wait to play stale.
            if !runtime.is_dormant() {
                for sounds in [&effects.play_start, &effects.play_end]
                    .into_iter()
                    .flatten()
                {
                    play_remote_sounds(runtime, sounds, listener, speaker_distance);
                }
            }
            runtime.send_mute_changes(server, mute);
        }
        PluginCommand::ReleaseAllTangents { nickname } => {
            state.lock().session.apply_release_all_tangents(&nickname);
        }
        PluginCommand::Volume {
            nickname,
            voice_volume,
            is_talking,
        } => {
            let mute = {
                let mut state = state.lock();
                let Some(applied) =
                    state
                        .session
                        .apply_remote_volume(&nickname, voice_volume, is_talking, now)
                else {
                    return;
                };
                (!applied.is_own_client)
                    .then(|| {
                        decide_client_mute(&mut state, &settings, applied.client_id, None, now)
                    })
                    .flatten()
            };
            runtime.send_mute_changes(server, mute);
        }
        PluginCommand::RequestVolume { client_id } => {
            let reply = {
                let state = state.lock();
                state.session.clients.own().map(|own| {
                    encode(&PluginCommand::Volume {
                        nickname: own.nickname().to_owned(),
                        voice_volume: state.session.game.voice_volume,
                        is_talking: own.is_talking_now,
                    })
                })
            };
            if let (Some(reply), Some(ts)) = (reply, runtime.act()) {
                let target = [host_client_id(client_id)];
                ts.send_plugin_command(server, &reply, PluginCommandTarget::Clients(&target));
            }
        }
    }
}

/// Queues one remote tangent sound per radio that heard it.
fn play_remote_sounds(
    runtime: &PluginRuntime,
    sounds: &RemoteRadioSounds,
    listener: Option<RemoteSoundListener>,
    speaker_distance: f64,
) {
    for placement in &sounds.placements {
        let plan = RemoteSoundPlan {
            placement: placement.clone(),
            listener,
            speaker_distance,
        };
        runtime.sounds.play_remote(sounds.sound, &plan);
    }
}

/// One of the own client's variables changed: when the talking flag changes in game, the own
/// talk state is recorded and `VOLUME` goes to the channel.
pub fn own_variable_changed(
    runtime: &PluginRuntime,
    server: ServerConnectionHandlerId,
    flag: i32,
    new_value: &str,
) {
    let is_talking_flag = i32::try_from(ClientProperty::FlagTalking.flag()).ok() == Some(flag);
    if !is_talking_flag || !runtime.is_in_game() {
        return;
    }
    let is_talking = new_value == "1";
    let Some(state) = runtime.servers.get(server) else {
        return;
    };
    let command = {
        let mut state = state.lock();
        let voice_volume = state.session.game.voice_volume;
        let Some(own) = state.session.clients.own_mut() else {
            return;
        };
        own.is_talking_now = is_talking;
        encode(&PluginCommand::Volume {
            nickname: own.nickname().to_owned(),
            voice_volume,
            is_talking,
        })
    };
    if let Some(ts) = runtime.act() {
        let front = ts.current_server();
        ts.send_plugin_command(front, &command, PluginCommandTarget::CurrentChannel);
    }
}
