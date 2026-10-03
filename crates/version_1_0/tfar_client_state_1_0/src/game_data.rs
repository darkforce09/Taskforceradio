//! `GameData`: what the game reports about the local player, and the speaker radios.
//!
//! **Role:** hold the local player's radios, voice range, liveness, environment values, the
//! loudspeaker radios, the data-frame counter and the local push-to-talk state, as `gameData`
//! (`reference/tfar_1_0/ts/src/task_force_radio.hpp:44-93`).
//!
//! **Position:** a module of `tfar_client_state_1_0`; part of [`crate::SessionState`]; written by
//! the session's `FREQ`, `SPEAKERS`, `DFRAME` and `KILLED` handling and by the plugin app's PTT
//! code; read by the routing crate and the voice pipeline.
//!
//! **Signals and state:** plain data behind the caller's session lock; the game link worker
//! writes it, the audio thread reads it.
//!
//! **Invariants:** the defaults are the original's (`myVoiceVolume` 0, not alive, waves 0,
//! terrain coefficient 7, global volume 1, receiving multiplier 1, speaker distance 20, frame
//! invalid); a frequency map holds the last entry the game listed for each frequency.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use tfar_arma_values::FrequencyEntry;
use tfar_game_commands_1_0::{FrequencyReport, VehicleDescriptor};
use tfar_geometry::Position;
use tfar_plugin_commands_1_0::TeamspeakClientId;

use crate::data_frame::DataFrame;

#[cfg(test)]
#[path = "tests/game_data.rs"]
mod tests;

/// One radio playing over a loudspeaker (`SPEAKER_DATA`, `task_force_radio.hpp:21-42`).
#[derive(Clone, Debug, PartialEq)]
pub struct Speaker {
    /// The radio's id, the key of its effects; shared, so a reception names it without copying.
    pub radio_id: Arc<str>,
    /// The owner's TeamSpeak client, when the owner's nickname was known as the speakers
    /// arrived; the owner's position stands in for a speaker without one.
    pub owner: Option<TeamspeakClientId>,
    /// Where the speaker is; `None` when it is on its owner.
    pub position: Option<Position>,
    /// The radio volume.
    pub volume: f64,
    /// The vehicle the speaker is in; shared, so a reception names it without copying.
    pub vehicle: Arc<VehicleDescriptor>,
    /// The height that decides whether the speaker plays under water.
    pub wave_z: f64,
}

/// What the game reports about the local player (`gameData`).
#[derive(Clone, Debug, PartialEq)]
pub struct GameData {
    /// The short-range radios the player has on, by frequency.
    pub sw_frequencies: BTreeMap<String, FrequencyEntry>,
    /// The long-range radios the player has on, by frequency.
    pub lr_frequencies: BTreeMap<String, FrequencyEntry>,
    /// The loudspeaker radios, by frequency, in the order the game listed them.
    pub speakers: BTreeMap<String, Vec<Speaker>>,
    /// The player's voice range in metres (`myVoiceVolume`).
    pub voice_volume: i32,
    /// The player is alive.
    pub is_alive: bool,
    /// The sea state, 0 to 1.
    pub waves_level: f64,
    /// How strongly terrain blocks radio waves (`terrainIntersectionCoefficient`).
    pub terrain_interception_coefficient: f64,
    /// The volume of everything the player hears.
    pub global_volume: f64,
    /// The receiving range multiplier (`receivingDistanceMultiplicator`).
    pub receiving_distance_multiplier: f64,
    /// How far loudspeaker radios carry, in metres.
    pub speaker_distance: f64,
    /// The game's current data frame.
    pub current_data_frame: DataFrame,
    /// The class name of the radio the player transmits on, kept in half-duplex mode only;
    /// `None` when not transmitting.
    pub current_transmitting_radio: Option<String>,
    /// The player holds a radio tangent.
    pub is_tangent_pressed: bool,
    /// TeamSpeak's push-to-talk release delay, read once from its settings database.
    pub push_to_talk_delay: Duration,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            sw_frequencies: BTreeMap::new(),
            lr_frequencies: BTreeMap::new(),
            speakers: BTreeMap::new(),
            voice_volume: 0,
            is_alive: false,
            waves_level: 0.0,
            terrain_interception_coefficient: 7.0,
            global_volume: 1.0,
            receiving_distance_multiplier: 1.0,
            speaker_distance: 20.0,
            current_data_frame: DataFrame::INVALID,
            current_transmitting_radio: None,
            is_tangent_pressed: false,
            push_to_talk_delay: Duration::ZERO,
        }
    }
}

impl GameData {
    /// Takes the values of a `FREQ` message (`gameData::setFreqInfos`): both frequency maps, the
    /// liveness, the voice range and the environment values.
    pub fn apply_frequency_report(&mut self, report: &FrequencyReport) {
        self.sw_frequencies = by_frequency(&report.sw_frequencies);
        self.lr_frequencies = by_frequency(&report.lr_frequencies);
        self.is_alive = report.is_alive;
        self.voice_volume = report.voice_volume;
        self.waves_level = report.waves_level;
        self.terrain_interception_coefficient = report.terrain_interception_coefficient;
        self.global_volume = report.global_volume;
        self.receiving_distance_multiplier = report.receiving_distance_multiplier;
        self.speaker_distance = report.speaker_distance;
    }

    /// Advances the data frame (`DFRAME`); the invalid frame stays invalid.
    pub const fn next_data_frame(&mut self) {
        self.current_data_frame = self.current_data_frame.next();
    }

    /// The speakers playing `frequency`, in the order the game listed them.
    pub fn speakers_on(&self, frequency: &str) -> &[Speaker] {
        self.speakers.get(frequency).map_or(&[], Vec::as_slice)
    }

    /// The game link came up: a new game starts at the first frame (`onGameConnected`).
    pub const fn start_game(&mut self) {
        self.current_data_frame = DataFrame::FIRST;
    }

    /// The game link went down: the player is no longer alive and the frame is invalid
    /// (`reference/tfar_1_0/ts/src/plugin.cpp:338-341`).
    pub const fn end_game(&mut self) {
        self.is_alive = false;
        self.current_data_frame = DataFrame::INVALID;
    }
}

/// The entries keyed by frequency; a later entry for a frequency replaces an earlier one, as the
/// original's `std::map` assignment does.
fn by_frequency(entries: &[FrequencyEntry]) -> BTreeMap<String, FrequencyEntry> {
    entries
        .iter()
        .map(|entry| (entry.frequency.clone(), entry.clone()))
        .collect()
}
