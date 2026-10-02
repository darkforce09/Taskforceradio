//! `POS` and `IS_SPEAKING`: a player's position and abilities, and whether they talk.
//!
//! **Role:** apply a `PositionUpdate` to the local player's entry or another player's
//! (`processUnitPosition`, `reference/tfar_0_9_12/ts/src/plugin.cpp:1580-1678`) and give what
//! the `SPEAKING` reply needs.
//!
//! **Position:** methods of `ServerRadioData`, called by the plugin's game link thread with the
//! parsed `tfar_game_commands_0_9::PositionUpdate`.
//!
//! **Signals and state:** writes one entry and the local player's position, view angle and
//! can-speak flag; time comes in as `now`.
//!
//! **Invariants:** a `POS` for the local player sets the view angle from field 5, one for
//! another player never touches theirs but sets the local player's from field 13; an unknown
//! nickname changes no entry and asks for the client list, as the original refreshes it first.

use std::time::Instant;

use tfar_game_commands_0_9::PositionUpdate;

use crate::client_data::ClientData;
use crate::client_id::ClientId;
use crate::server_radio_data::ServerRadioData;

#[cfg(test)]
#[path = "tests/positions.rs"]
mod tests;

/// What a `POS` did, and what the plugin does next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionOutcome {
    /// The local player's own position: the plugin centres the 3D listener. `speaking` is
    /// `None` when the local player has no entry yet, which answers `NOT_SPEAKING`.
    Mine {
        /// The talking state for the reply.
        speaking: Option<SpeakingState>,
    },
    /// Another player's position: the plugin updates their mute state and centres their 3D
    /// position.
    Other {
        /// The talking state for the reply.
        speaking: SpeakingState,
    },
    /// The nickname is in no entry: the plugin refreshes the channel client list
    /// ([`ServerRadioData::update_nicknames`]) and applies the `POS` again; if the nickname is
    /// still unknown it answers `NOT_SPEAKING`.
    UnknownNickname,
}

/// What the `SPEAKING` reply needs: the reply is `SPEAKING` when TeamSpeak's talking flag of
/// `client_id` is set or `is_talking_on_radio` holds, else `NOT_SPEAKING`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpeakingState {
    /// The player's client id, whose TeamSpeak talking flag the plugin reads.
    pub client_id: ClientId,
    /// Whether the player holds a push-to-talk or talks by their last `VOLUME`.
    pub is_talking_on_radio: bool,
}

impl ServerRadioData {
    /// Applies `update` at `now`. `my_client_id` is the local player's client id, which the
    /// local player's entry takes; `None` when TeamSpeak cannot tell it.
    pub fn apply_position(
        &mut self,
        update: &PositionUpdate,
        my_client_id: Option<ClientId>,
        now: Instant,
    ) -> PositionOutcome {
        let data_frame = self.current_data_frame;
        if update.nickname == self.my_nickname {
            self.my_position = update.position;
            self.can_speak = update.can_speak;
            let speaking = self.my_client_mut().map(|mine| {
                if let Some(client_id) = my_client_id {
                    mine.client_id = client_id;
                }
                write_position(mine, update, data_frame, now);
                mine.view_angle = update.view_angle;
                speaking_state(mine)
            });
            return PositionOutcome::Mine { speaking };
        }
        if let Some(mine) = self.my_client_mut() {
            mine.view_angle = update.current_direction;
        }
        match self.clients.get_mut(&update.nickname) {
            Some(other) => {
                write_position(other, update, data_frame, now);
                PositionOutcome::Other {
                    speaking: speaking_state(other),
                }
            }
            None => PositionOutcome::UnknownNickname,
        }
    }

    /// What the `SPEAKING` reply to `IS_SPEAKING` needs for the player `nickname`; `None` for
    /// an unknown nickname, which answers `NOT_SPEAKING`.
    pub fn speaking_state_of(&self, nickname: &str) -> Option<SpeakingState> {
        self.clients.get(nickname).map(speaking_state)
    }
}

/// The fields every `POS` writes, the view angle excepted.
fn write_position(client: &mut ClientData, update: &PositionUpdate, data_frame: i32, now: Instant) {
    client.position = update.position;
    client.position_time = Some(now);
    client.can_speak = update.can_speak;
    client.usable_radios = update.usable_radios;
    client.vehicle.clone_from(&update.vehicle);
    client.terrain_interception = update.terrain_interception;
    client.data_frame = data_frame;
    client.voice_volume_multiplier = update.voice_volume_multiplier;
}

/// The talking state of `client`.
fn speaking_state(client: &ClientData) -> SpeakingState {
    SpeakingState {
        client_id: client.client_id,
        is_talking_on_radio: client.is_talking_on_radio(),
    }
}
