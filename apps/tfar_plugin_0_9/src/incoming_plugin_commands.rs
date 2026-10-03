//! The plugin commands other players' TFAR plugins send: their push-to-talk, the release of all
//! of it, and their voice volume.
//!
//! **Role:** `ts3plugin_onPluginCommandEvent` and `processPluginCommand`
//! (`reference/tfar_0_9_12/ts/src/plugin.cpp:2940-3130`): take a command only on the current
//! server and only from a `task_force_radio` plugin, decode it, apply it to the sender's entry,
//! decide the sender's mute for each reception, make the sound state their voice now needs, and
//! queue the remote start or end sound per reception while the listener lives.
//!
//! **Position:** methods of `PluginCore`, called from `onPluginCommandEvent` on TeamSpeak's
//! main thread.
//!
//! **Signals and state:** reads the sender's talking flag and channel first, then one short lock
//! of the server state to decide; the mute requests and the sounds follow with no lock held.
//!
//! **Invariants:** a dormant plugin applies the command to its state but sends no mute request
//! and plays no sound; a remote push-to-talk plays its sound only on a flip of the sender's held
//! state; a phone or unknown subtype plays nothing.

use std::path::PathBuf;
use std::time::Instant;

use tfar_arma_values::RadioSubtype;
use tfar_client_state_0_9::{ClientId as StateClientId, VehicleMuffling};
use tfar_client_state_0_9::{RemoteTangent, ServerRadioData, TangentSound};
use tfar_plugin_commands_0_9::{PluginCommand as DecodedCommand, TangentKind, accepts_sender};
use tfar_radio_routing_0_9::{
    HeardOn, HeardVia, IgnoreTangents, MuteDecision, MuteQuery, receptions,
    set_game_client_mute_status,
};
use tfar_ts3_host::{ClientId, PluginCommand, ServerConnectionHandlerId};
use tfar_voice_pipeline_0_9::{
    ListenerView, RemoteSoundPlan, radio_volume_multiplier, render_remote_tangent_sound,
};

use crate::game_messages::game_tangent_kind;
use crate::plugin_core::{MuteFacts, PluginCore};
use crate::radio_sounds::{RadioSounds, TangentMoment};
use crate::server_connection::ServerState;

/// One remote push-to-talk sound to play: the WAV file, decoded at init, and how the voice
/// pipeline shapes it.
#[derive(Clone, Debug, PartialEq)]
pub struct PlannedRemoteSound {
    /// The WAV file in `radio-sounds/`.
    pub path: PathBuf,
    /// The gain, place and filters `render_remote_tangent_sound` applies.
    pub plan: RemoteSoundPlan,
}

#[cfg(test)]
#[path = "tests/incoming_plugin_commands.rs"]
mod tests;

/// An incoming push-to-talk, decoded.
struct IncomingTangent<'command> {
    kind: TangentKind,
    is_pressed: bool,
    frequency: &'command str,
    range: i32,
    subtype: &'command RadioSubtype,
    nickname: &'command str,
}

impl PluginCore {
    /// Handles `command` from another client's plugin at `now`.
    pub fn handle_plugin_command(&self, command: &PluginCommand<'_>, now: Instant) {
        let server = command.server;
        if server != self.team_speak().current_server() {
            tracing::debug!("a plugin command from a server tab not in front is ignored");
            return;
        }
        if !accepts_sender(command.plugin_name) {
            tracing::debug!(
                plugin = command.plugin_name,
                "a foreign plugin command is ignored"
            );
            return;
        }
        let decoded = match DecodedCommand::decode(command.command) {
            Ok(decoded) => decoded,
            Err(error) => {
                tracing::debug!(%error, "a plugin command was refused");
                return;
            }
        };
        let sender = command.invoker.id;
        match decoded {
            DecodedCommand::Tangent {
                kind,
                pressed,
                frequency,
                range,
                subtype,
                nickname,
            } => {
                let tangent = IncomingTangent {
                    kind,
                    is_pressed: pressed,
                    frequency: &frequency,
                    range,
                    subtype: &subtype,
                    nickname: &nickname,
                };
                self.apply_remote_tangent(server, sender, &tangent, now);
            }
            DecodedCommand::ReleaseAllTangents { nickname } => {
                self.with_state(server, |state| {
                    state.radio.apply_release_all_tangents(&nickname);
                    state.ensure_sender_dsp(&nickname, self.hrir());
                });
            }
            DecodedCommand::Volume {
                nickname,
                voice_volume,
                is_talking,
            } => self.apply_remote_volume(server, sender, &nickname, voice_volume, is_talking, now),
        }
    }

    /// A remote push-to-talk: state, mute per reception, sound state, and the sounds.
    fn apply_remote_tangent(
        &self,
        server: ServerConnectionHandlerId,
        sender: ClientId,
        tangent: &IncomingTangent<'_>,
        now: Instant,
    ) {
        let facts = self.mute_facts_of(server, sender);
        let is_dormant = self.is_dormant();
        let decided = self.with_state(server, |state| {
            let kind = game_tangent_kind(tangent.kind);
            let outcome = state.radio.apply_remote_tangent(
                &RemoteTangent {
                    kind,
                    is_pressed: tangent.is_pressed,
                    frequency: tangent.frequency,
                    range: tangent.range,
                    subtype: tangent.subtype,
                    nickname: tangent.nickname,
                },
                now,
            );
            let mut decision = MuteDecision::default();
            let mut sounds = Vec::new();
            if let Some(outcome) = outcome
                && outcome.has_fresh_data
                && let Some(me) = state.my_client()
            {
                let heard = receptions(
                    &state.radio,
                    tangent.nickname,
                    me,
                    IgnoreTangents::for_kind(kind),
                );
                if !heard.is_empty() && !is_dormant {
                    decision = decide_mute(state, outcome.client_id, me, &facts);
                }
                if let Some(sound) = outcome
                    .sound
                    .filter(|_| outcome.is_listener_alive && !is_dormant)
                {
                    sounds = heard
                        .iter()
                        .filter_map(|reception| {
                            plan_remote_sound(
                                self.sounds(),
                                &state.radio,
                                me,
                                reception,
                                tangent.subtype,
                                sound,
                            )
                        })
                        .collect();
                }
            }
            state.ensure_sender_dsp(tangent.nickname, self.hrir());
            (decision, sounds)
        });
        let Some((decision, sounds)) = decided else {
            return;
        };
        self.send_mute_decision(server, &decision);
        for sound in &sounds {
            self.queue_remote_sound(server, sound);
        }
    }

    /// A remote `VOLUME`: state, the sender's mute, and their sound state.
    fn apply_remote_volume(
        &self,
        server: ServerConnectionHandlerId,
        sender: ClientId,
        nickname: &str,
        voice_volume: i32,
        is_talking: bool,
        now: Instant,
    ) {
        let facts = self.mute_facts_of(server, sender);
        let is_dormant = self.is_dormant();
        let decision = self
            .with_state(server, |state| {
                let outcome =
                    state
                        .radio
                        .apply_remote_volume(nickname, voice_volume, is_talking, now);
                let decision = match (outcome, state.my_client()) {
                    (Some(outcome), Some(me))
                        if !outcome.is_mine && outcome.has_fresh_data && !is_dormant =>
                    {
                        Some(decide_mute(state, outcome.client_id, me, &facts))
                    }
                    _ => None,
                };
                state.ensure_sender_dsp(nickname, self.hrir());
                decision
            })
            .flatten();
        if let Some(decision) = decision {
            self.send_mute_decision(server, &decision);
        }
    }

    /// Renders `sound` from its decoded WAV and queues it in the mixer of `server`.
    fn queue_remote_sound(&self, server: ServerConnectionHandlerId, sound: &PlannedRemoteSound) {
        let Some(connection) = self.connection(server) else {
            return;
        };
        let Some(wave) = self.waves().cached(&sound.path) else {
            tracing::debug!(path = %sound.path.display(), "a remote radio sound is not loaded");
            return;
        };
        match render_remote_tangent_sound(&wave, &sound.plan, self.hrir()) {
            Ok(playback) => {
                if let Err(error) = connection.mixer.add(playback) {
                    tracing::debug!(%error, "a remote radio sound was dropped");
                }
            }
            Err(error) => tracing::warn!(%error, "a remote radio sound could not be prepared"),
        }
    }
}

/// The serious-mode mute of `client` as the listener `me` decides it, through the tracker.
fn decide_mute(
    state: &mut ServerState,
    client: StateClientId,
    me: StateClientId,
    facts: &MuteFacts,
) -> MuteDecision {
    let query = MuteQuery {
        client_id: client,
        my_client_id: me,
        is_in_serious_channel: facts.is_in_serious_channel(&state.radio.serious_channel_name),
        is_talking_in_teamspeak: facts.is_talking_in_teamspeak,
    };
    set_game_client_mute_status(&mut state.radio, &mut state.mutes, &query)
}

/// The sound one reception of a remote push-to-talk plays (plugin.cpp:3018-3044); `None` for a
/// subtype without sounds.
fn plan_remote_sound(
    sounds: &RadioSounds,
    radio: &ServerRadioData,
    me: StateClientId,
    reception: &HeardVia,
    subtype: &RadioSubtype,
    sound: TangentSound,
) -> Option<PlannedRemoteSound> {
    let moment = match sound {
        TangentSound::Start => TangentMoment::RemoteStart,
        TangentSound::End => TangentMoment::RemoteEnd,
    };
    let path = sounds.tangent(subtype, moment)?;
    let listener = radio
        .fresh_client_by_id(me)
        .map(|(_, listener)| ListenerView {
            position: radio.my_position,
            view_angle: listener.view_angle,
            vehicle_isolation: VehicleMuffling::between(&listener.vehicle, &reception.vehicle)
                .muffling_isolation(),
        });
    Some(PlannedRemoteSound {
        path,
        plan: RemoteSoundPlan {
            gain: radio_volume_multiplier(reception.volume) * radio.global_volume,
            position: reception.position,
            is_on_loudspeaker: reception.on == HeardOn::Loudspeaker,
            radio_volume: reception.volume,
            is_under_water: reception.is_under_water(),
            speaker_distance: radio.speaker_distance,
            listener,
        },
    })
}
