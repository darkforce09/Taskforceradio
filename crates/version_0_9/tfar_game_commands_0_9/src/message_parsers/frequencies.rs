//! `FREQ`: the local player's radios, nickname and sound settings.
//!
//! **Role:** read the 13 fields of a `FREQ` message into a `FrequencyUpdate`.
//!
//! **Position:** a message parser of `tfar_game_commands_0_9`, called by `GameCommand::parse`;
//! follows the `FREQ` branch of `processGameCommand` in `reference/tfar_0_9_12/ts/src/plugin.cpp`.
//!
//! **Signals and state:** none; a pure function.
//!
//! **Invariants:** exactly 14 fields; field 11 is never read; the voice volume rounds and the
//! diver volume level truncates as the original computes them; a malformed number or frequency
//! list is an error naming its field.

use tfar_arma_values::{FrequencyEntry, FrequencyEntryShape, parse_frequency_entries};

use crate::command_names::FREQ;
use crate::error::{Error, Result};
use crate::field_values::{exact_fields, is_true, number, rounded_integer, truncated_integer};
use crate::game_command::GameCommand;

#[cfg(test)]
#[path = "tests/frequencies.rs"]
mod tests;

/// The fields of a `FREQ` message, typed, as `processGameCommand` in
/// `reference/tfar_0_9_12/ts/src/plugin.cpp` stores them. Field 11, the voice volume, is not
/// here: the original reads the voice volume from field 5 and never reads field 11, so it is
/// neither parsed nor checked.
#[derive(Clone, Debug, PartialEq)]
pub struct FrequencyUpdate {
    /// Field 1: the short-range radios on, in list order. The plugin keys them by frequency, so a
    /// later entry for the same frequency wins.
    pub sw: Vec<FrequencyEntry>,
    /// Field 2: the long-range radios on, in list order.
    pub lr: Vec<FrequencyEntry>,
    /// Field 3: the diver radio frequency, as sent.
    pub dd_frequency: String,
    /// Field 4: whether the local player is alive (`"true"`).
    pub alive: bool,
    /// Field 5: the distance in metres the local voice carries, rounded.
    pub my_voice_volume: i32,
    /// Field 6: the diver radio volume level, truncated as `(int)atof` truncates it.
    pub dd_volume_level: i32,
    /// Field 7: the nickname the game wants the local player to carry in TeamSpeak.
    pub nickname: String,
    /// Field 8: the sea state, which drowns diver radios.
    pub waves_level: f64,
    /// Field 9: the terrain interception coefficient.
    pub terrain_coefficient: f64,
    /// Field 10: the global volume factor.
    pub global_volume: f64,
    /// Field 12: the receiving distance multiplier.
    pub receiving_distance_multiplier: f64,
    /// Field 13: the distance in metres a loudspeaker radio carries.
    pub speaker_distance: f64,
}

/// `FREQ` and 13 fields: exactly 14.
pub(crate) fn parse_frequencies(fields: &[&str]) -> Result<GameCommand> {
    let [
        _,
        sw,
        lr,
        dd_frequency,
        alive,
        my_voice_volume,
        dd_volume_level,
        nickname,
        waves_level,
        terrain_coefficient,
        global_volume,
        _ignored_voice_volume,
        receiving_distance_multiplier,
        speaker_distance,
    ] = exact_fields::<14>(FREQ, fields)?;
    Ok(GameCommand::Frequencies(FrequencyUpdate {
        sw: frequency_list("sw", sw)?,
        lr: frequency_list("lr", lr)?,
        dd_frequency: dd_frequency.to_owned(),
        alive: is_true(alive),
        my_voice_volume: rounded_integer(FREQ, "my_voice_volume", my_voice_volume)?,
        dd_volume_level: truncated_integer(FREQ, "dd_volume_level", dd_volume_level)?,
        nickname: nickname.to_owned(),
        waves_level: number(FREQ, "waves_level", waves_level)?,
        terrain_coefficient: number(FREQ, "terrain_coefficient", terrain_coefficient)?,
        global_volume: number(FREQ, "global_volume", global_volume)?,
        receiving_distance_multiplier: number(
            FREQ,
            "receiving_distance_multiplier",
            receiving_distance_multiplier,
        )?,
        speaker_distance: number(FREQ, "speaker_distance", speaker_distance)?,
    }))
}

/// The entries of a frequency list field: `frequency|volume|stereo` strings, entries of another
/// part count skipped (`parseFrequencies`).
fn frequency_list(field: &'static str, text: &str) -> Result<Vec<FrequencyEntry>> {
    parse_frequency_entries(text, FrequencyEntryShape::ThreeParts).map_err(|source| {
        Error::Malformed {
            command: FREQ,
            field,
            source,
        }
    })
}
