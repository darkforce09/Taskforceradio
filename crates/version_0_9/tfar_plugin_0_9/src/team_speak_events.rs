//! The TeamSpeak events the 0.9 plugin follows: connection status, channel and client changes,
//! our own talking flag, the info panel and server errors.
//!
//! **Role:** `onConnectStatusChangeEvent` (state made on established, with TeamSpeak's own 3D
//! neutralised, and dropped on disconnect), the channel and client events that refresh the
//! nickname list (`reference/tfar_0_9_12/ts/src/plugin.cpp:2398-2460`), a client leaving (its
//! mute forgotten), our talking flag (the `VOLUME` broadcast, plugin.cpp:2897), a client starting
//! to talk or stopping (`onTalkStatusChangeEvent`: their mute decided again at once, and on a
//! start their HRTF direction sources reset), `infoData` (the
//! `<TFAR>` block of a client's metadata) and `onServerErrorEvent`.
//!
//! **Position:** methods of `PluginCore`, called from the `Ts3Plugin` event methods of
//! `TfarPlugin` on TeamSpeak's main thread.
//!
//! **Signals and state:** each event reads TeamSpeak first and then takes the connection's
//! state lock briefly; the broadcast is sent after the lock is released.
//!
//! **Invariants:** a client that leaves the server is forgotten by the mute tracker, so its id,
//! when TeamSpeak gives it to someone else, is never unmuted by mistake; a dormant plugin
//! broadcasts nothing.

use tfar_plugin_commands_0_9::{PluginCommand, info_data_text};
use tfar_radio_routing_0_9::{MuteQuery, set_game_client_mute_status};
use tfar_ts3_host::{
    ChannelId, ClientId, ClientMove, ClientProperty, ConnectStatus, ErrorDisplay, InfoItem,
    ServerConnectionHandlerId, ServerError, TalkStatus,
};

use crate::channel_snapshot::state_client_id;
use crate::plugin_core::PluginCore;

#[cfg(test)]
#[path = "tests/team_speak_events.rs"]
mod tests;

/// The new value of a talking flag that means "talking".
const TALKING_VALUE: &str = "1";

impl PluginCore {
    /// The connection of `server` reached `status`.
    pub fn handle_connect_status(&self, server: ServerConnectionHandlerId, status: ConnectStatus) {
        match status {
            ConnectStatus::ConnectionEstablished => {
                self.add_connection(server);
                if let Err(error) = self.team_speak().neutralise_3d_settings(server) {
                    tracing::warn!(%error, "TeamSpeak's 3D settings could not be set");
                }
            }
            ConnectStatus::Disconnected => self.remove_connection(server),
            _ => {}
        }
    }

    /// `client` on `server` started or stopped talking (`onTalkStatusChangeEvent`): their mute is
    /// decided again at once, and when they start their HRTF direction sources are reset, so the
    /// tail of their last utterance never replays.
    pub fn handle_talk_status_change(
        &self,
        server: ServerConnectionHandlerId,
        status: TalkStatus,
        client: ClientId,
    ) {
        let is_talking = status.is_talking();
        tracing::info!(client = ?client, is_talking, "talk status: changed");
        if self.is_dormant() {
            return;
        }
        // The mute is decided again at once: a player muted between utterances is unmuted on
        // the event that says they speak, not on their next `POS` or `VOLUME`, which would cut
        // the start of every sentence.
        let facts = self.mute_facts_of(server, client);
        let sender = state_client_id(client);
        let decision = self
            .with_state(server, |state| {
                if is_talking {
                    state.reset_direction_sources_of(sender);
                }
                // As for `VOLUME` (plugin.cpp:3099), only a player with current game data is
                // decided again: without it (a spectator, a player the game no longer reports)
                // the rule would mute them on every word.
                if !state.radio.has_fresh_data(sender) {
                    return None;
                }
                let me = state.my_client().filter(|me| *me != sender)?;
                let query = MuteQuery {
                    client_id: sender,
                    my_client_id: me,
                    is_in_serious_channel: facts
                        .is_in_serious_channel(&state.radio.serious_channel_name),
                    is_talking_in_teamspeak: is_talking || facts.is_talking_in_teamspeak,
                };
                Some(set_game_client_mute_status(
                    &mut state.radio,
                    &mut state.mutes,
                    &query,
                ))
            })
            .flatten();
        if let Some(decision) = decision {
            self.send_mute_decision(server, &decision);
        }
    }

    /// A channel or client changed on `server`: the nickname list and the voice facts are read
    /// again.
    pub fn handle_channel_change(&self, server: ServerConnectionHandlerId) {
        self.refresh_channel(server);
    }

    /// A client moved, timed out or was kicked: the list is read again, and a client that left
    /// the server is forgotten.
    pub fn handle_client_move(&self, movement: &ClientMove<'_>) {
        if movement.new_channel == ChannelId(0) {
            let client = state_client_id(movement.client);
            self.with_state(movement.server, |state| {
                state.mutes.forget(client);
                state.facts.forget(client);
            });
        }
        self.refresh_channel(movement.server);
    }

    /// One of our own variables changed: a change of the talking flag in game broadcasts our
    /// voice volume and whether we talk (`VOLUME`).
    pub fn handle_own_variable_update(
        &self,
        server: ServerConnectionHandlerId,
        flag: i32,
        new_value: &str,
    ) {
        let is_talking_flag =
            usize::try_from(flag).ok() == Some(ClientProperty::FlagTalking.flag());
        if !is_talking_flag || !self.is_in_game() || self.is_dormant() {
            return;
        }
        let team_speak = self.team_speak();
        let current = team_speak.current_server();
        let nickname = team_speak
            .my_client_id(current)
            .and_then(|me| team_speak.client_nickname(current, me))
            .unwrap_or_default();
        let Some(voice_volume) = self.with_state(current, |state| state.radio.my_voice_volume)
        else {
            return;
        };
        let command = PluginCommand::Volume {
            nickname,
            voice_volume,
            is_talking: new_value == TALKING_VALUE,
        }
        .encode();
        if let Err(error) = team_speak.send_plugin_command(current, &command) {
            tracing::warn!(%error, ?server, "the voice volume was not broadcast");
        }
    }

    /// The text of TeamSpeak's info panel for `item`: a client's `<TFAR>` status block.
    pub fn info_text(&self, server: ServerConnectionHandlerId, item: InfoItem) -> Option<String> {
        let InfoItem::Client(client) = item else {
            return None;
        };
        let metadata = self.team_speak().client_metadata(server, client)?;
        info_data_text(&metadata).map(str::to_owned)
    }
}

/// `onServerErrorEvent`: an error answering a request that carried a return code is handled
/// here and not shown (plugin.cpp:2468).
pub fn server_error_display(error: &ServerError<'_>) -> ErrorDisplay {
    if error.return_code.is_empty() {
        ErrorDisplay::Show
    } else {
        ErrorDisplay::Suppress
    }
}
