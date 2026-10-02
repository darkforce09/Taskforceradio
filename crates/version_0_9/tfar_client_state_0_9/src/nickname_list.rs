//! The channel client list: which nickname carries which TeamSpeak client id.

use crate::client_data::ClientData;
use crate::client_id::ClientId;
use crate::server_radio_data::ServerRadioData;

#[cfg(test)]
#[path = "tests/nickname_list.rs"]
mod tests;

impl ServerRadioData {
    /// Takes in the clients of the local player's channel as `(client id, nickname)` pairs
    /// (`updateNicknamesList`, `reference/tfar_0_9_12/ts/src/plugin.cpp:1478`): an unknown
    /// nickname gets a fresh entry, a known one takes the id. Entries of clients not listed stay
    /// until they expire.
    pub fn update_nicknames(&mut self, channel_clients: &[(ClientId, String)]) {
        for (client_id, nickname) in channel_clients {
            self.clients
                .entry(nickname.clone())
                .or_insert_with(|| ClientData::new(*client_id))
                .client_id = *client_id;
        }
    }

    /// Records the local player's TeamSpeak nickname, which the original reads after each
    /// client list update (plugin.cpp:1508).
    pub fn set_my_nickname(&mut self, my_nickname: &str) {
        if self.my_nickname != my_nickname {
            my_nickname.clone_into(&mut self.my_nickname);
        }
    }
}
