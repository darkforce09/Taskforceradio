//! The entry point of every game message: parse, apply, start the game, reply.
//!
//! **Role:** `processGameCommand` and the pipe thread around it
//! (`reference/tfar_0_9_12/ts/src/plugin.cpp:1769-1936`, 2056-2071): parse the message with
//! `tfar_plugin_logic_0_9::game_commands`, apply it to the current server's state, perform its TeamSpeak side
//! effects with no lock held, start the game on the first message that counts as game activity,
//! and answer with the reply string the original sends.
//!
//! **Position:** methods of `PluginCore`, called by the game link bridge on the game link's
//! session thread; the position, push-to-talk and frequency messages have their own files.
//!
//! **Signals and state:** locks one server state at a time for each decision; the game session
//! mutex only in `record_game_activity`, before the command is applied; a `TRACK` request is
//! kept in the core's web requests for shutdown to join.
//!
//! **Invariants:** every message gets exactly one reply; a malformed or short message is `FAIL`
//! with a log line, never a crash; a message that counts as game activity wakes a dormant plugin
//! before its side effects run, and `VERSION` never does.

use std::time::Instant;

use tfar_plugin_logic_0_9::client_state::SeriousChannel;
use tfar_plugin_logic_0_9::game_commands::replies::{
    DONE, ERROR_GETTING_SERVER_NAME, FAIL, NO_CHANNEL_NAME, NOT_SPEAKING, OK, PONG, SPEAKING,
};
use tfar_plugin_logic_0_9::game_commands::{
    GameCommand, TsInfoQuery, message_counts_as_game_activity,
};
use tfar_plugin_logic_0_9::plugin_commands::PluginCommand;
use tfar_plugin_logic_0_9::radio_routing::{
    MuteDecision, is_serious_mode, set_mute_for_dead_players,
};
use tfar_ts3_host::ServerConnectionHandlerId;
use tfar_web_requests::{build_track_request_0_9, spawn_cancellable_track};

use crate::channel_snapshot::read_channel_snapshot;
use crate::plugin_core::PluginCore;

#[cfg(test)]
#[path = "tests/message_dispatch.rs"]
mod tests;

impl PluginCore {
    /// Handles the game message `text` at `now` on the current server and returns the reply.
    pub fn handle_game_message(&self, text: &str, now: Instant) -> String {
        let starts_a_game = self.record_game_activity(message_counts_as_game_activity(text), now);
        let server = self.team_speak().current_server();
        let reply = match GameCommand::parse(text) {
            Ok(command) => self.apply_game_command(server, command, now),
            Err(error) => {
                tracing::debug!(%error, "a game message was refused");
                error.reply().to_owned()
            }
        };
        if starts_a_game {
            self.start_game(server);
        }
        reply
    }

    /// Applies one parsed command on `server` and returns its reply.
    fn apply_game_command(
        &self,
        server: ServerConnectionHandlerId,
        command: GameCommand,
        now: Instant,
    ) -> String {
        match command {
            GameCommand::TsInfo(query) => self.answer_ts_info(server, &query),
            GameCommand::Killed { nickname } => self.apply_killed(server, &nickname),
            GameCommand::Speakers(records) => {
                self.with_state(server, |state| {
                    state.radio.apply_speakers(&records);
                    state.ensure_every_sender_dsp(self.hrir());
                });
                OK.to_owned()
            }
            GameCommand::ReleaseAllTangents { nickname } => {
                let command = PluginCommand::ReleaseAllTangents { nickname }.encode();
                if let Err(error) = self.team_speak().send_plugin_command(server, &command) {
                    tracing::warn!(%error, "the release of every push-to-talk was not sent");
                }
                OK.to_owned()
            }
            GameCommand::Version {
                addon_version,
                serious_channel_name,
                serious_channel_password,
            } => {
                self.apply_version(
                    server,
                    &addon_version,
                    &serious_channel_name,
                    &serious_channel_password,
                );
                OK.to_owned()
            }
            GameCommand::Track { path } => {
                self.send_tracking_request(&path);
                OK.to_owned()
            }
            GameCommand::Position(update) => self.apply_position_message(server, &update, now),
            GameCommand::Tangent {
                kind,
                pressed,
                frequency,
                range,
                subtype,
            } => self
                .apply_local_tangent_message(server, kind, pressed, frequency, range, subtype, now),
            GameCommand::Frequencies(update) => self.apply_frequency_message(server, &update),
            GameCommand::IsSpeaking { nickname } => self.answer_is_speaking(server, &nickname),
            GameCommand::Unknown(_) => FAIL.to_owned(),
        }
    }

    /// `TS_INFO` (`ts_info`, plugin.cpp:1534): the server name, our channel's name, or `PONG`.
    fn answer_ts_info(&self, server: ServerConnectionHandlerId, query: &TsInfoQuery) -> String {
        let team_speak = self.team_speak();
        match query {
            TsInfoQuery::Server => team_speak
                .server_name(server)
                .unwrap_or_else(|| ERROR_GETTING_SERVER_NAME.to_owned()),
            TsInfoQuery::Channel => team_speak
                .my_client_id(server)
                .and_then(|me| team_speak.channel_of_client(server, me))
                .and_then(|channel| team_speak.channel_name(server, channel))
                .unwrap_or_else(|| NO_CHANNEL_NAME.to_owned()),
            TsInfoQuery::Ping => PONG.to_owned(),
            TsInfoQuery::Other(_) => FAIL.to_owned(),
        }
    }

    /// `VERSION` (plugin.cpp:1789): the handshake is stored and the data frame advances. A game
    /// that started before the mission named its serious channel makes the move at the first
    /// `VERSION` that names one, and a new addon version reaches the metadata at once.
    fn apply_version(
        &self,
        server: ServerConnectionHandlerId,
        addon_version: &str,
        serious_channel_name: &str,
        serious_channel_password: &str,
    ) {
        let is_in_game = self.is_in_game();
        let applied = self.with_state(server, |state| {
            let is_new_addon_version = state.radio.addon_version != addon_version;
            state.radio.apply_version(
                addon_version,
                serious_channel_name,
                serious_channel_password,
            );
            let pending_move =
                (is_in_game && state.is_serious_move_pending && !serious_channel_name.is_empty())
                    .then(|| {
                        state.is_serious_move_pending = false;
                        SeriousChannel {
                            name: serious_channel_name.to_owned(),
                            password: serious_channel_password.to_owned(),
                        }
                    });
            (is_new_addon_version, pending_move)
        });
        let Some((is_new_addon_version, pending_move)) = applied else {
            return;
        };
        if let Some(serious_channel) = pending_move {
            self.move_into_serious_channel(server, &serious_channel);
        }
        if is_new_addon_version && is_in_game {
            self.publish_status_metadata(server);
        }
    }

    /// `KILLED` (`processUnitKilled`, plugin.cpp:1562): the player's data stops being current,
    /// then the players without current data are muted while the local player lives in serious
    /// mode.
    fn apply_killed(&self, server: ServerConnectionHandlerId, nickname: &str) -> String {
        let snapshot = read_channel_snapshot(self.team_speak(), server);
        let decision = self.with_state(server, |state| {
            state.radio.apply_killed(nickname);
            let is_serious =
                is_serious_mode(&snapshot.channel_name, &state.radio.serious_channel_name);
            match state.my_client() {
                Some(me) => set_mute_for_dead_players(
                    &state.radio,
                    &mut state.mutes,
                    &snapshot.member_ids(),
                    me,
                    is_serious,
                ),
                None => MuteDecision::default(),
            }
        });
        if let Some(decision) = decision {
            self.send_mute_decision(server, &decision);
        }
        DONE.to_owned()
    }

    /// `IS_SPEAKING`: whether TeamSpeak's talking flag of the player is set, they hold a
    /// push-to-talk, or their last `VOLUME` said they talk.
    fn answer_is_speaking(&self, server: ServerConnectionHandlerId, nickname: &str) -> String {
        let speaking = self
            .with_state(server, |state| state.radio.speaking_state_of(nickname))
            .flatten();
        speaking_reply(self.is_speaking(server, speaking)).to_owned()
    }
}

/// `SPEAKING` or `NOT_SPEAKING`.
pub const fn speaking_reply(is_speaking: bool) -> &'static str {
    if is_speaking { SPEAKING } else { NOT_SPEAKING }
}

impl PluginCore {
    /// `TRACK`: the Piwik request for `path`, sent on its own thread and kept for shutdown to
    /// cancel and join; a failure is logged.
    fn send_tracking_request(&self, path: &str) {
        match build_track_request_0_9(path).and_then(spawn_cancellable_track) {
            Ok(request) => self.web_requests().keep(request),
            Err(error) => tracing::debug!(%error, "the tracking request was not sent"),
        }
    }
}
