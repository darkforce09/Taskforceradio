//! `FREQ`: the local player's radios and levels, and the rename to the game's nickname.

use tfar_game_commands_0_9::FrequencyUpdate;
use tfar_game_commands_0_9::replies::OK;
use tfar_ts3_host::ServerConnectionHandlerId;

use crate::plugin_core::PluginCore;

impl PluginCore {
    /// Applies `FREQ` on `server` (`reference/tfar_0_9_12/ts/src/plugin.cpp:1874-1908`): the
    /// radios and levels are stored and every sender's sound state remade; when the game names
    /// the local player differently, TeamSpeak's nickname is set to the game's, the first
    /// rename of the session saves the original, and the channel is read again so the state
    /// knows the new name. Returns `OK`.
    pub fn apply_frequency_message(
        &self,
        server: ServerConnectionHandlerId,
        update: &FrequencyUpdate,
    ) -> String {
        let team_speak = self.team_speak();
        let current_nickname = team_speak
            .my_client_id(server)
            .and_then(|me| team_speak.client_nickname(server, me))
            .unwrap_or_default();
        let rename = self
            .with_state(server, |state| {
                let rename = state.radio.apply_frequencies(update, &current_nickname);
                state.ensure_every_sender_dsp(self.hrir());
                rename
            })
            .flatten();
        if let Some(request) = rename {
            match team_speak.set_own_nickname(server, &request.game_nickname) {
                Ok(()) => {
                    self.with_state(server, |state| state.radio.confirm_rename(&request));
                    self.refresh_channel(server);
                }
                Err(error) => {
                    tracing::warn!(%error, "the nickname could not be set to the game's");
                }
            }
        }
        OK.to_owned()
    }
}
