//! `POS`: a player's position and abilities, the mute decision it drives, and the `SPEAKING`
//! reply.
//!
//! **Role:** `processUnitPosition` (`reference/tfar_0_9_12/ts/src/plugin.cpp:1580-1678`) on the
//! plugin side: apply the update, refresh the client list once for an unknown nickname, decide
//! the player's mute in serious mode, make the sound state their voice now needs, neutralise
//! TeamSpeak's own 3D position, and answer whether the player talks.
//!
//! **Position:** a method of `PluginCore`, called by the message dispatch on the game link
//! thread.
//!
//! **Signals and state:** three short locks of the server state (apply, decide, and the second
//! apply after a refresh); the TeamSpeak reads (our id, the talking flag, the channel name) and
//! writes (mutes, the 3D calls) happen between them, with no lock held.
//!
//! **Invariants:** the local player's own `POS` centres the 3D listener and remakes every
//! sender's sound state; another player's `POS` decides only that player's mute and sound state.

use std::time::Instant;

use tfar_client_state_0_9::{PositionOutcome, SpeakingState};
use tfar_game_commands_0_9::PositionUpdate;
use tfar_game_commands_0_9::replies::NOT_SPEAKING;
use tfar_radio_routing_0_9::{MuteQuery, set_game_client_mute_status};
use tfar_ts3_host::ServerConnectionHandlerId;

use super::message_dispatch::speaking_reply;
use crate::channel_snapshot::{host_client_id, state_client_id};
use crate::plugin_core::PluginCore;

impl PluginCore {
    /// Applies `POS` at `now` on `server` and returns `SPEAKING` or `NOT_SPEAKING`.
    pub fn apply_position_message(
        &self,
        server: ServerConnectionHandlerId,
        update: &PositionUpdate,
        now: Instant,
    ) -> String {
        let team_speak = self.team_speak();
        let my_client = team_speak.my_client_id(server).map(state_client_id);
        let apply = || {
            self.with_state(server, |state| {
                state.radio.apply_position(update, my_client, now)
            })
        };
        let mut outcome = apply();
        if outcome == Some(PositionOutcome::UnknownNickname) {
            self.refresh_channel(server);
            outcome = apply();
        }
        match outcome {
            Some(PositionOutcome::Mine { speaking }) => {
                self.with_state(server, |state| state.ensure_every_sender_dsp(self.hrir()));
                if let Err(error) = team_speak.centre_listener(server) {
                    tracing::debug!(%error, "the 3D listener could not be centred");
                }
                speaking_reply(self.is_speaking(server, speaking)).to_owned()
            }
            Some(PositionOutcome::Other { speaking }) => {
                self.decide_mute_after_position(server, &update.nickname, speaking)
            }
            Some(PositionOutcome::UnknownNickname) | None => NOT_SPEAKING.to_owned(),
        }
    }

    /// The mute, the sound state and the 3D position of the other player `nickname` after their
    /// `POS`; returns the reply.
    fn decide_mute_after_position(
        &self,
        server: ServerConnectionHandlerId,
        nickname: &str,
        speaking: SpeakingState,
    ) -> String {
        let team_speak = self.team_speak();
        let client = host_client_id(speaking.client_id);
        let facts = self.mute_facts_of(server, client);
        let decision = self
            .with_state(server, |state| {
                let decision = state.my_client().map(|me| {
                    let query = MuteQuery {
                        client_id: speaking.client_id,
                        my_client_id: me,
                        is_in_serious_channel: facts
                            .is_in_serious_channel(&state.radio.serious_channel_name),
                        is_talking_in_teamspeak: facts.is_talking_in_teamspeak,
                    };
                    set_game_client_mute_status(&mut state.radio, &mut state.mutes, &query)
                });
                state.ensure_sender_dsp(nickname, self.hrir());
                decision
            })
            .flatten();
        if let Some(decision) = decision {
            self.send_mute_decision(server, &decision);
        }
        if let Err(error) = team_speak.centre_client(server, client) {
            tracing::debug!(%error, "a client's 3D position could not be centred");
        }
        speaking_reply(facts.is_talking_in_teamspeak || speaking.is_talking_on_radio).to_owned()
    }

    /// Whether the player of `speaking` talks: TeamSpeak's flag or the radio state.
    pub fn is_speaking(
        &self,
        server: ServerConnectionHandlerId,
        speaking: Option<SpeakingState>,
    ) -> bool {
        speaking.is_some_and(|speaking| {
            speaking.is_talking_on_radio
                || self
                    .team_speak()
                    .is_client_talking(server, host_client_id(speaking.client_id))
        })
    }
}
