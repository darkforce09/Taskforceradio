//! The plugin commands other players' plugins send: their push-to-talk, its release and their
//! voice volume.
//!
//! **Role:** apply the three incoming plugin commands to the sender's entry
//! (`processTangentPress`, `processAllTangentRelease` and the `VOLUME` branch of
//! `processPluginCommand`, `reference/tfar_0_9_12/ts/src/plugin.cpp:2940-3110`).
//!
//! **Position:** methods of `ServerRadioData`, called by the plugin on TeamSpeak's
//! plugin-command event with the decoded `tfar_plugin_commands_0_9::PluginCommand` fields; the
//! plugin then runs the routing and the mute decision on what they return.
//!
//! **Signals and state:** writes one entry; a released push-to-talk heard by a living listener
//! drops the sender's radio effects.
//!
//! **Invariants:** a command for an unknown nickname changes nothing; a push-to-talk asks for
//! the start or end sound only when it flips the sender's held state.

use std::time::Instant;

use tfar_game_commands_0_9::TangentKind;
use tfar_game_commands_0_9::prelude::RadioSubtype;

use crate::client_id::ClientId;
use crate::server_radio_data::ServerRadioData;

#[cfg(test)]
#[path = "tests/plugin_commands.rs"]
mod tests;

/// One incoming push-to-talk, as the `TANGENT` plugin command carries it.
#[derive(Clone, Debug, PartialEq)]
pub struct RemoteTangent<'command> {
    /// The radio family, from the command name.
    pub kind: TangentKind,
    /// Whether the push-to-talk is now pressed.
    pub is_pressed: bool,
    /// The frequency with the radio code appended.
    pub frequency: &'command str,
    /// The radio's range in metres.
    pub range: i32,
    /// The kind of radio.
    pub subtype: &'command RadioSubtype,
    /// The nickname of the player who keyed the radio.
    pub nickname: &'command str,
}

/// What an incoming push-to-talk asks the plugin to do next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoteTangentOutcome {
    /// The sender's client id.
    pub client_id: ClientId,
    /// Whether the sender has current data: only then does the plugin route the push-to-talk
    /// (with the ignore flags of its kind), update the sender's mute state for each reception
    /// and play the sound.
    pub has_fresh_data: bool,
    /// The sound the flip asks for, played per reception while the listener is alive.
    pub sound: Option<TangentSound>,
    /// Whether the local listener was alive when the command arrived.
    pub is_listener_alive: bool,
    /// Whether the sender's radio effects were dropped (a release heard by a living listener).
    pub dropped_radio_effects: bool,
}

/// The remote push-to-talk sound a flip asks for (`radio-sounds/<family>/remote_start` or
/// `remote_end`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TangentSound {
    /// The push-to-talk went down.
    Start,
    /// The push-to-talk went up.
    End,
}

/// What an incoming `VOLUME` asks the plugin to do next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoteVolumeOutcome {
    /// The sender's client id.
    pub client_id: ClientId,
    /// Whether the sender is the local player, whose own mute state never changes.
    pub is_mine: bool,
    /// Whether the sender has current data: with it, and not mine, the plugin updates the
    /// sender's mute state.
    pub has_fresh_data: bool,
}

impl ServerRadioData {
    /// Applies an incoming push-to-talk at `now`; `None` for an unknown nickname.
    pub fn apply_remote_tangent(
        &mut self,
        tangent: &RemoteTangent<'_>,
        now: Instant,
    ) -> Option<RemoteTangentOutcome> {
        let is_listener_alive = self.is_alive;
        let sender = self.clients.get_mut(tangent.nickname)?;
        sender.position_time = Some(now);
        sender.is_plugin_enabled = true;
        sender.plugin_enabled_check = Some(now);
        sender.subtype.clone_from(tangent.subtype);
        sender.allow_radio(tangent.kind);
        let sound =
            (sender.tangent.is_some() != tangent.is_pressed).then_some(if tangent.is_pressed {
                TangentSound::Start
            } else {
                TangentSound::End
            });
        sender.tangent = tangent.is_pressed.then_some(tangent.kind);
        tangent.frequency.clone_into(&mut sender.frequency);
        sender.range = tangent.range;
        let client_id = sender.client_id;
        let has_fresh_data = self.has_fresh_data(client_id);
        let dropped_radio_effects =
            has_fresh_data && is_listener_alive && sound == Some(TangentSound::End);
        if dropped_radio_effects && let Some(sender) = self.clients.get_mut(tangent.nickname) {
            sender.reset_radio_effects();
        }
        Some(RemoteTangentOutcome {
            client_id,
            has_fresh_data,
            sound,
            is_listener_alive,
            dropped_radio_effects,
        })
    }

    /// Applies an incoming `RELEASE_ALL_TANGENTS`: the player `nickname` holds no push-to-talk.
    /// Returns whether the nickname is known.
    pub fn apply_release_all_tangents(&mut self, nickname: &str) -> bool {
        self.clients
            .get_mut(nickname)
            .map(|client| client.tangent = None)
            .is_some()
    }

    /// Applies an incoming `VOLUME` at `now`: the player's voice range and talking state, and a
    /// sign that their plugin runs; `None` for an unknown nickname.
    pub fn apply_remote_volume(
        &mut self,
        nickname: &str,
        voice_volume: i32,
        is_talking: bool,
        now: Instant,
    ) -> Option<RemoteVolumeOutcome> {
        let is_mine = nickname == self.my_nickname;
        let sender = self.clients.get_mut(nickname)?;
        sender.voice_volume = voice_volume;
        sender.is_plugin_enabled = true;
        sender.plugin_enabled_check = Some(now);
        sender.is_talking_now = is_talking;
        let client_id = sender.client_id;
        Some(RemoteVolumeOutcome {
            client_id,
            is_mine,
            has_fresh_data: self.has_fresh_data(client_id),
        })
    }
}
