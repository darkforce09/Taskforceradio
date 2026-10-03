//! The start and end of a game session: the on and off sounds, the serious-mode channel move
//! and its way back, the unmute and the nickname given back.
//!
//! **Role:** `onGameStart` and `onGameEnd` with the pipe and service threads' transitions around
//! them (`reference/tfar_0_9_12/ts/src/plugin.cpp:1391-1476`, 1992-2009, 2056-2071), over the
//! decisions `tfar_client_state_0_9` makes (`ServerRadioData::start_game`, `end_game`).
//!
//! **Position:** methods of `PluginCore`; the game link thread starts a game after the first
//! message that counts as game activity, the service thread ends it on the 4 s timeout and
//! puts the plugin back to dormant, and `ts3plugin_shutdown` ends a running one.
//!
//! **Signals and state:** writes the in-game flag and the connection's serious-mode return
//! channel; every TeamSpeak call is made with no lock held.
//!
//! **Invariants:** serious mode moves only into a channel of exactly the mission's name, and only
//! when the player is not in it already; the way back goes to the channel the move left; the
//! nickname given back is the player's own, saved at the first rename of the session; after a
//! game timed out, every side effect of its end is done before the plugin is dormant again.

use tfar_client_state_0_9::SeriousChannel;
use tfar_radio_routing_0_9::is_other_radio_plugin_channel;
use tfar_ts3_host::ServerConnectionHandlerId;

use crate::plugin_core::PluginCore;

impl PluginCore {
    /// A game starts on `server`: the on sound, data frame 0, the serious-mode move when the
    /// mission names a channel, and the new status in the metadata.
    pub fn start_game(&self, server: ServerConnectionHandlerId) {
        self.set_in_game(true);
        self.play_sound(server, &self.sounds().game_start());
        let start = self.with_state(server, |state| {
            let start = state.radio.start_game();
            state.is_serious_move_pending = start.serious_channel.is_none();
            state.ensure_every_sender_dsp(self.hrir());
            start
        });
        if let Some(serious_channel) = start.and_then(|start| start.serious_channel) {
            self.move_into_serious_channel(server, &serious_channel);
        }
        self.publish_status_metadata(server);
    }

    /// The game ends on `server`: a push-to-talk release still waiting cancelled (voice
    /// activation and the microphone restored, no broadcast), the off sound when
    /// `plays_end_sound`, the local player dead and outside any data frame, the way back from the
    /// serious channel, every mute this plugin made lifted, the nickname given back, and the new
    /// status in the metadata.
    pub fn end_game(&self, server: ServerConnectionHandlerId, plays_end_sound: bool) {
        self.set_in_game(false);
        self.push_to_talk()
            .cancel_pending_release(self.team_speak());
        if plays_end_sound {
            self.play_sound(server, &self.sounds().game_end());
        }
        let ended = self.with_state(server, |state| {
            state.is_serious_move_pending = false;
            (
                state.radio.end_game(),
                state.channel_before_serious_mode.take(),
            )
        });
        let Some((end, channel_before)) = ended else {
            return;
        };
        let team_speak = self.team_speak();
        if let Some(channel) = channel_before
            && let Some(me) = team_speak.my_client_id(server)
            && team_speak.channel_of_client(server, me) != Some(channel)
            && let Err(error) = team_speak.move_client(server, me, channel, "")
        {
            tracing::warn!(%error, "could not move back out of the serious channel");
        }
        self.unmute_everyone(server);
        if let Some(nickname) = end.restore_nickname {
            match team_speak.set_own_nickname(server, &nickname) {
                Ok(()) => {
                    self.refresh_channel(server);
                }
                Err(error) => tracing::warn!(%error, "could not set the nickname back"),
            }
        }
        self.publish_status_metadata(server);
    }

    /// The game went silent for 4 s: a running game ends with the off sound and the plugin
    /// returns to dormant until the next game activity; without one, the mutes this plugin made
    /// are lifted unless the local player sits in another radio plugin's channel, where that
    /// plugin owns the mutes.
    pub fn time_out_game(&self, server: ServerConnectionHandlerId, ended_a_game: bool) {
        if ended_a_game {
            self.end_game(server, true);
            self.return_to_dormant();
            return;
        }
        let is_in_other_radio_channel = self
            .with_state(server, |state| {
                let me = state.my_client();
                me.is_some_and(|me| is_other_radio_plugin_channel(state.facts.channel_name_of(me)))
            })
            .unwrap_or(false);
        if !is_in_other_radio_channel {
            self.unmute_everyone(server);
        }
    }

    /// Lifts every mute this plugin made on `server`.
    pub fn unmute_everyone(&self, server: ServerConnectionHandlerId) {
        if let Some(decision) = self.with_state(server, |state| state.mutes.unmute_all()) {
            self.send_mute_decision(server, &decision);
        }
    }

    /// Moves the local player into the channel named `serious_channel.name` and remembers where
    /// they came from.
    pub fn move_into_serious_channel(
        &self,
        server: ServerConnectionHandlerId,
        serious_channel: &SeriousChannel,
    ) {
        let team_speak = self.team_speak();
        let Some(me) = team_speak.my_client_id(server) else {
            return;
        };
        let before = team_speak.channel_of_client(server, me);
        let target = team_speak.channel_list(server).into_iter().find(|channel| {
            team_speak.channel_name(server, *channel).as_deref()
                == Some(serious_channel.name.as_str())
        });
        let Some(target) = target else {
            tracing::info!(name = %serious_channel.name, "the serious mode channel does not exist");
            return;
        };
        if before == Some(target) {
            return;
        }
        match team_speak.move_client(server, me, target, &serious_channel.password) {
            Ok(()) => {
                self.with_state(server, |state| state.channel_before_serious_mode = before);
            }
            Err(error) => tracing::warn!(%error, "could not join the serious mode channel"),
        }
    }
}
