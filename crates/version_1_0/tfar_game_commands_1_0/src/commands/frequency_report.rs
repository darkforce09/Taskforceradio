//! `FrequencyReport`: the `FREQ` message, the local player's radios and state.
//!
//! **Role:** read the ten fields of `FREQ` into a typed report, rejecting a wrong token count or
//! a malformed field.
//!
//! **Position:** a module of the crate's `commands`; built on `MessageFields` and
//! `tfar_arma_values::parse_frequency_entries`; called by the parser.
//!
//! **Signals and state:** none; pure parsing.
//!
//! **Invariants:** the fields are those `gameData::setFreqInfos` reads
//! (`reference/tfar_1_0/ts/src/task_force_radio.hpp:47-69`) at the same indices, the message is
//! exactly 11 tokens, `alive` is true only for exactly `true`, and frequency entries have three or
//! four parts (`helpers::parseFrequencies`, `helpers.cpp:307-326`).

use tfar_arma_values::{FrequencyEntry, FrequencyEntryShape, parse_frequency_entries};

use crate::error::Result;
use crate::fields::MessageFields;

#[cfg(test)]
#[path = "tests/frequency_report.rs"]
mod tests;

/// The nicknames the plugin never adopts as the player's TeamSpeak name
/// (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:255`).
pub const NICKNAMES_NEVER_ADOPTED: [&str; 3] = ["Error: No unit", "Error: No vehicle", "any"];

/// One `FREQ` message: the local player's radios and the values that shape what they hear.
#[derive(Clone, Debug, PartialEq)]
pub struct FrequencyReport {
    /// The short-range radios, in list order; a later entry for the same frequency replaces an
    /// earlier one, as the original's map does.
    pub sw_frequencies: Vec<FrequencyEntry>,
    /// The long-range radios, in list order, with the same rule.
    pub lr_frequencies: Vec<FrequencyEntry>,
    /// The local player is alive (the field is exactly `true`).
    pub is_alive: bool,
    /// The local player's voice range in metres, rounded (`myVoiceVolume`).
    pub voice_volume: i32,
    /// The local player's in-game name, sanitized as the plugin compares it.
    pub nickname: String,
    /// The sea state, 0 to 1.
    pub waves_level: f64,
    /// How strongly terrain blocks radio waves.
    pub terrain_interception_coefficient: f64,
    /// The volume of everything the player hears.
    pub global_volume: f64,
    /// The receiving range multiplier; smaller means a longer range.
    pub receiving_distance_multiplier: f64,
    /// How far loudspeaker radios carry, in metres.
    pub speaker_distance: f64,
}

impl FrequencyReport {
    /// The nickname the plugin gives the player in TeamSpeak, or `None` when it is one of
    /// [`NICKNAMES_NEVER_ADOPTED`]. The plugin also skips the rename when the name is already
    /// the player's.
    pub fn nickname_to_adopt(&self) -> Option<&str> {
        (!NICKNAMES_NEVER_ADOPTED.contains(&self.nickname.as_str())).then_some(&self.nickname)
    }
}

/// The `FREQ` message whose tokens are `fields`.
pub(crate) fn parse_frequency_report(fields: &MessageFields<'_>) -> Result<FrequencyReport> {
    fields.require_count(11)?;
    let entries = |index: usize, field: &'static str| {
        fields.read(index, field, |text| {
            parse_frequency_entries(text, FrequencyEntryShape::ThreeOrFourParts).ok()
        })
    };
    Ok(FrequencyReport {
        sw_frequencies: entries(1, "short-range frequencies")?,
        lr_frequencies: entries(2, "long-range frequencies")?,
        is_alive: fields.text(3)? == "true",
        voice_volume: fields.integer(4, "voice volume")?,
        nickname: fields.nickname(5)?,
        waves_level: fields.number(6, "waves level")?,
        terrain_interception_coefficient: fields.number(7, "terrain interception coefficient")?,
        global_volume: fields.number(8, "global volume")?,
        receiving_distance_multiplier: fields.number(9, "receiving distance multiplier")?,
        speaker_distance: fields.number(10, "speaker distance")?,
    })
}
