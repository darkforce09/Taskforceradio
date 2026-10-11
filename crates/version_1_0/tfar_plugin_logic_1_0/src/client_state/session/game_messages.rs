//! The apply functions of the game's messages: `POS`, `FREQ`, `KILLED` and `SPEAKERS`.
//!
//! **Role:** write each parsed game message into the session and say what the plugin app must
//! do in TeamSpeak afterwards.
//!
//! **Position:** a module of the crate's `session`; called by the plugin app's game link
//! handler with the payloads of `tfar_plugin_logic_1_0::game_commands`.
//!
//! **Signals and state:** changes the session's game data and clients; the game link worker
//! calls it under the session lock.
//!
//! **Invariants:** the fields go where `reference/tfar_1_0/ts/src/CommandProcessor.cpp` puts
//! them; an unknown nickname changes no client.

use std::sync::Arc;
use std::time::Instant;

use crate::game_commands::{FrequencyReport, PositionUpdate, SpeakerRecord};
use crate::plugin_commands::TeamspeakClientId;

use crate::client_state::client_data::ClientData;
use crate::client_state::data_frame::DataFrame;
use crate::client_state::game_data::Speaker;
use crate::client_state::session::session_state::SessionState;

#[cfg(test)]
#[path = "tests/game_messages.rs"]
mod tests;

/// What [`SessionState::apply_position`] changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PositionApplied {
    /// The client whose position arrived.
    pub client_id: TeamspeakClientId,
    /// The client is the local player: the app neutralises TeamSpeak's listener position;
    /// otherwise it recomputes the client's mute and neutralises its 3D position
    /// (`CommandProcessor.cpp:580-596`).
    pub is_own_client: bool,
}

/// A rename of the local player's TeamSpeak nickname that a `FREQ` message asks for.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenameRequest {
    /// The local player's client.
    pub client_id: TeamspeakClientId,
    /// The game name to adopt. Once TeamSpeak has renamed the client, the app reports it with
    /// [`crate::client_state::ClientDirectory::client_renamed`], as the original fires
    /// `onTeamspeakClientUpdated`.
    pub nickname: String,
}

/// What [`SessionState::apply_killed`] changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KilledApplied {
    /// The client killed, when known.
    pub killed: Option<TeamspeakClientId>,
    /// The local player is still alive afterwards; the app runs the killed-player mute matrix
    /// with it.
    pub is_player_alive: bool,
}

impl SessionState {
    /// Takes a `POS` message at `now` (`clientData::updatePosition`, `clientData.cpp:76-102`).
    /// The radio abilities are taken from the local player's own message only, as the others'
    /// do not say whether they carry that radio. `None` for an unknown nickname.
    pub fn apply_position(
        &mut self,
        update: &PositionUpdate,
        now: Instant,
    ) -> Option<PositionApplied> {
        let current_frame = self.game.current_data_frame;
        let own_client_id = self.clients.own_client_id();
        let client = self.clients.get_mut(&update.nickname)?;
        let is_own_client = own_client_id == Some(client.client_id());
        client.position = update.position;
        client.view_direction = update.view_direction;
        client.can_speak = update.can_speak;
        if is_own_client {
            client.can_use_sw_radio = update.can_use_sw_radio;
            client.can_use_lr_radio = update.can_use_lr_radio;
            client.can_use_dd_radio = update.can_use_dd_radio;
        }
        if *client.vehicle != update.vehicle {
            client.vehicle = Arc::new(update.vehicle.clone());
        }
        client.terrain_interception = update.terrain_interception;
        client.voice_volume_multiplier = update.voice_volume;
        client.object_interception = update.object_interception;
        client.is_spectating = update.is_spectating;
        client.is_enemy_to_player = update.is_enemy_to_player;
        client.position_updated_at = Some(now);
        client.last_activity_at = Some(now);
        client.data_frame = current_frame;
        Some(PositionApplied {
            client_id: client.client_id(),
            is_own_client,
        })
    }

    /// Takes a `FREQ` message (`CommandProcessor.cpp:236-260`) and returns the rename it asks
    /// for: the game name when it is not one the plugin never adopts, differs from the local
    /// player's nickname and that nickname is known and not empty.
    pub fn apply_frequencies(&mut self, report: &FrequencyReport) -> Option<RenameRequest> {
        self.game.apply_frequency_report(report);
        let own = self.clients.own()?;
        let nickname = report.nickname_to_adopt()?;
        if own.nickname().is_empty() || own.nickname() == nickname {
            return None;
        }
        Some(RenameRequest {
            client_id: own.client_id(),
            nickname: nickname.to_owned(),
        })
    }

    /// Takes a `KILLED` message (`processUnitKilled`, `CommandProcessor.cpp:531-551`): the
    /// client's data frame becomes invalid, and the local player is dead when its own frame is
    /// invalid.
    pub fn apply_killed(&mut self, nickname: &str) -> KilledApplied {
        let killed = self.clients.get_mut(nickname).map(|client| {
            client.data_frame = DataFrame::INVALID;
            client.client_id()
        });
        if self
            .clients
            .own()
            .is_some_and(|own| !own.data_frame.is_valid())
        {
            self.game.is_alive = false;
        }
        KilledApplied {
            killed,
            is_player_alive: self.game.is_alive,
        }
    }

    /// Takes a `SPEAKERS` message (`processSpeakers`, `CommandProcessor.cpp:493-529`): the
    /// speakers are replaced, each listed under every frequency it plays, with its owner found
    /// by nickname now.
    pub fn apply_speakers(&mut self, records: &[SpeakerRecord]) {
        self.game.speakers.clear();
        for record in records {
            let speaker = Speaker {
                radio_id: Arc::from(record.radio_id.as_str()),
                owner: self
                    .clients
                    .get(&record.nickname)
                    .map(ClientData::client_id),
                position: record.position,
                volume: f64::from(record.volume),
                vehicle: Arc::new(record.vehicle.clone()),
                wave_z: record.wave_z,
            };
            for frequency in &record.frequencies {
                self.game
                    .speakers
                    .entry(frequency.clone())
                    .or_default()
                    .push(speaker.clone());
            }
        }
    }
}
