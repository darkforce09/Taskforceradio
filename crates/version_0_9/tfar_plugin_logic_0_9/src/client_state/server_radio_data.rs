//! `ServerRadioData`: everything the 0.9.12 plugin knows on one server connection.
//!
//! **Role:** hold every field of the original's `SERVER_RADIO_DATA`
//! (`reference/tfar_0_9_12/ts/src/plugin.cpp:402-440`): the local player's identity, radios and
//! settings, the players by nickname, the loudspeaker radios and the data frame.
//!
//! **Position:** the root of `tfar_plugin_logic_0_9::client_state`; the plugin app owns one per server
//! connection behind one lock, the command application in `command_application/` writes it, and
//! the routing and voice pipeline read it.
//!
//! **Signals and state:** plain fields. The plugin holds it in one `parking_lot::Mutex` per
//! connection; the game link and TeamSpeak event threads lock it to apply a command, the audio
//! thread `try_lock`s it and passes a frame through when the lock is busy.
//!
//! **Invariants:** every field starts at a defined value (the original leaves the position,
//! the alive and can-speak flags and the levels uninitialised until the first `POS` or `FREQ`);
//! the players are ordered by nickname, as the original's `std::map`; a frequency maps to at
//! most one setting, the last one the game listed.

use std::collections::BTreeMap;

use crate::game_commands::prelude::StereoMode;
use tfar_foundation::geometry::Position;

use crate::client_state::client_data::ClientData;
use crate::client_state::client_id::ClientId;
use crate::client_state::speaker_radio::SpeakerRadio;

#[cfg(test)]
#[path = "tests/server_radio_data.rs"]
mod tests;

/// The data frame outside a game (`INVALID_DATA_FRAME`, plugin.cpp:77); no player's frame lies
/// within one of it once a game has started at frame 0.
pub const INVALID_DATA_FRAME: i32 = 9999;

/// The volume and ear of one radio the local player has on (`FREQ_SETTINGS`, plugin.cpp:370).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrequencySettings {
    /// The radio's volume.
    pub volume: i32,
    /// The ear the radio plays into.
    pub stereo_mode: StereoMode,
}

/// The state of one server connection.
#[derive(Debug)]
pub struct ServerRadioData {
    /// The local player's TeamSpeak nickname, from the channel client list.
    pub my_nickname: String,
    /// The nickname the local player had before the game first renamed them this game session;
    /// `None` when the game has not renamed them.
    pub my_original_nickname: Option<String>,
    /// Whether the local player holds a push-to-talk.
    pub is_tangent_pressed: bool,
    /// The local player's position (`POS` of the local player; the origin, as the game sends
    /// every position relative to the local listener).
    pub my_position: Position,
    /// Every player in the local player's channel, the local player included, by nickname.
    pub clients: BTreeMap<String, ClientData>,
    /// The local player's short-range radios on, by frequency.
    pub sw_frequencies: BTreeMap<String, FrequencySettings>,
    /// The local player's long-range radios on, by frequency.
    pub lr_frequencies: BTreeMap<String, FrequencySettings>,
    /// The local player's diver radio frequency.
    pub dd_frequency: String,
    /// The loudspeaker radios, by each frequency they listen on, in the order the game listed
    /// them.
    pub speakers: BTreeMap<String, Vec<SpeakerRadio>>,
    /// The diver radio volume level.
    pub dd_volume_level: i32,
    /// The distance in metres the local player's voice carries.
    pub my_voice_volume: i32,
    /// Whether the local player is alive.
    pub is_alive: bool,
    /// Whether the local player can speak.
    pub can_speak: bool,
    /// The sea state, which shortens diver radio range.
    pub waves_level: f64,
    /// The metres of distance each unit of terrain interception adds.
    pub terrain_intersection_coefficient: f64,
    /// The gain on everything the plugin outputs.
    pub global_volume: f64,
    /// The factor on every radio distance.
    pub receiving_distance_multiplier: f64,
    /// The distance in metres a loudspeaker radio at volume 10 carries.
    pub speaker_distance: f64,
    /// The channel serious mode moves players into; empty for none.
    pub serious_channel_name: String,
    /// The password of the serious mode channel.
    pub serious_channel_password: String,
    /// The version of the TFAR addon the game runs.
    pub addon_version: String,
    /// The current data frame: 0 at game start, one up per `VERSION`, [`INVALID_DATA_FRAME`]
    /// outside a game.
    pub current_data_frame: i32,
}

impl Default for ServerRadioData {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl ServerRadioData {
    /// The state of a fresh connection on which the local player is `my_nickname`, with the
    /// original's defaults: terrain coefficient 7, global volume and distance multiplier 1,
    /// loudspeaker distance 20, outside a game, not alive and able to speak.
    pub fn new(my_nickname: String) -> Self {
        Self {
            my_nickname,
            my_original_nickname: None,
            is_tangent_pressed: false,
            my_position: Position::ORIGIN,
            clients: BTreeMap::new(),
            sw_frequencies: BTreeMap::new(),
            lr_frequencies: BTreeMap::new(),
            dd_frequency: String::new(),
            speakers: BTreeMap::new(),
            dd_volume_level: 0,
            my_voice_volume: 0,
            is_alive: false,
            can_speak: true,
            waves_level: 0.0,
            terrain_intersection_coefficient: 7.0,
            global_volume: 1.0,
            receiving_distance_multiplier: 1.0,
            speaker_distance: 20.0,
            serious_channel_name: String::new(),
            serious_channel_password: String::new(),
            addon_version: String::new(),
            current_data_frame: INVALID_DATA_FRAME,
        }
    }

    /// The entry of the player `nickname`.
    pub fn client(&self, nickname: &str) -> Option<&ClientData> {
        self.clients.get(nickname)
    }

    /// The entry of the player `nickname`, to change.
    pub fn client_mut(&mut self, nickname: &str) -> Option<&mut ClientData> {
        self.clients.get_mut(nickname)
    }

    /// The local player's entry, by [`ServerRadioData::my_nickname`].
    pub fn my_client(&self) -> Option<&ClientData> {
        self.clients.get(&self.my_nickname)
    }

    /// The local player's entry, to change.
    pub fn my_client_mut(&mut self) -> Option<&mut ClientData> {
        self.clients.get_mut(&self.my_nickname)
    }

    /// The client id of the player `nickname` (`getClientId`, plugin.cpp:848).
    pub fn client_id_of(&self, nickname: &str) -> Option<ClientId> {
        self.clients.get(nickname).map(|client| client.client_id)
    }

    /// The loudspeaker radios that listen on `frequency`, in the order the game listed them.
    pub fn speakers_on(&self, frequency: &str) -> &[SpeakerRadio] {
        self.speakers.get(frequency).map_or(&[], Vec::as_slice)
    }
}
