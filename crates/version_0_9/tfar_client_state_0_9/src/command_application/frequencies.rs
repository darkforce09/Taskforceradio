//! `FREQ`: the local player's radios and settings, and the rename the game asks for.
//!
//! **Role:** store a `FrequencyUpdate` (the `FREQ` branch of `processGameCommand`,
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:1874-1908`) and decide whether the local player's
//! TeamSpeak nickname changes to the game's.
//!
//! **Position:** methods of `ServerRadioData`, called by the plugin's game link thread; the
//! plugin performs the rename and reports success with `confirm_rename`.
//!
//! **Signals and state:** writes the frequency maps and levels, and the original nickname.
//!
//! **Invariants:** a frequency listed twice keeps its last setting; the original nickname is
//! saved once per game session, at the first successful rename, so the nickname set back at
//! game end is the player's own and never a game name.

use std::collections::BTreeMap;
use tfar_game_commands_0_9::FrequencyUpdate;
use tfar_game_commands_0_9::prelude::FrequencyEntry;

use crate::server_radio_data::{FrequencySettings, ServerRadioData};

#[cfg(test)]
#[path = "tests/frequencies.rs"]
mod tests;

/// The nicknames the game sends when it has none to give, which the plugin never adopts
/// (plugin.cpp:1895).
pub const NICKNAMES_NEVER_ADOPTED: [&str; 3] = ["Error: No unit", "Error: No vehicle", "any"];

/// A rename of the local player the game asks for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenameRequest {
    /// The local player's TeamSpeak nickname now.
    pub current_nickname: String,
    /// The nickname the game gives the local player.
    pub game_nickname: String,
}

impl ServerRadioData {
    /// Stores `update`. `current_teamspeak_nickname` is the local player's nickname as
    /// TeamSpeak reports it now (empty when it cannot tell); the result asks the plugin to set
    /// the game's nickname when the two differ, the current one is known and the game's is a
    /// real name.
    pub fn apply_frequencies(
        &mut self,
        update: &FrequencyUpdate,
        current_teamspeak_nickname: &str,
    ) -> Option<RenameRequest> {
        self.sw_frequencies = frequency_map(&update.sw);
        self.lr_frequencies = frequency_map(&update.lr);
        self.dd_frequency.clone_from(&update.dd_frequency);
        self.is_alive = update.alive;
        self.my_voice_volume = update.my_voice_volume;
        self.dd_volume_level = update.dd_volume_level;
        self.waves_level = update.waves_level;
        self.terrain_intersection_coefficient = update.terrain_coefficient;
        self.global_volume = update.global_volume;
        self.receiving_distance_multiplier = update.receiving_distance_multiplier;
        self.speaker_distance = update.speaker_distance;
        let is_rename = current_teamspeak_nickname != update.nickname
            && !current_teamspeak_nickname.is_empty()
            && !NICKNAMES_NEVER_ADOPTED.contains(&update.nickname.as_str());
        is_rename.then(|| RenameRequest {
            current_nickname: current_teamspeak_nickname.to_owned(),
            game_nickname: update.nickname.clone(),
        })
    }

    /// Records that TeamSpeak accepted `request`: the nickname before it becomes the original
    /// nickname, unless an earlier rename of this game session saved one.
    pub fn confirm_rename(&mut self, request: &RenameRequest) {
        if self.my_original_nickname.is_none() {
            self.my_original_nickname = Some(request.current_nickname.clone());
        }
    }
}

/// The settings of `entries` by frequency, a later entry for a frequency replacing an earlier.
fn frequency_map(entries: &[FrequencyEntry]) -> BTreeMap<String, FrequencySettings> {
    entries
        .iter()
        .map(|entry| {
            (
                entry.frequency.clone(),
                FrequencySettings {
                    volume: whole_volume(entry.volume),
                    stereo_mode: entry.stereo_mode,
                },
            )
        })
        .collect()
}

/// The volume of an entry, which the parser rounded from an `i32`, back as an `i32`.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the parser stores a rounded i32 as f64, so the value is whole and in range"
)]
fn whole_volume(volume: f64) -> i32 {
    volume as i32
}
