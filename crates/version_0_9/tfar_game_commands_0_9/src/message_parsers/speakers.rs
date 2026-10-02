//! `SPEAKERS`: the radios on loudspeaker near the local player.
//!
//! **Role:** read the vertical-tab separated speaker records of a `SPEAKERS` message into typed
//! records.
//!
//! **Position:** a message parser of `tfar_game_commands_0_9`, called by `GameCommand::parse`;
//! follows `processSpeakers` in `reference/tfar_0_9_12/ts/src/plugin.cpp`.
//!
//! **Signals and state:** none; a pure function.
//!
//! **Invariants:** a message of 1 field, or of more than 2, is an empty list, which clears the
//! speakers as the original clears them; empty records are skipped (the first always is, as the
//! game writes a separator before each record); a record of fewer than 6 parts is an error, never
//! an out-of-bounds read.

use tfar_arma_values::separators::{FREQUENCY_PART, SPEAKER_PART, SPEAKER_RECORD};
use tfar_arma_values::{parse_optional_position, split_fields};
use tfar_geometry::Position;

use crate::command_names::SPEAKERS;
use crate::error::{Error, Result};
use crate::field_values::{number, rounded_integer};
use crate::game_command::GameCommand;
use crate::vehicle_descriptor::VehicleDescriptor;

#[cfg(test)]
#[path = "tests/speakers.rs"]
mod tests;

/// The wave height a record without its seventh part is given.
const DEFAULT_WAVE_Z: f64 = 1.0;

/// One radio playing on loudspeaker, from a `SPEAKERS` record.
#[derive(Clone, Debug, PartialEq)]
pub struct SpeakerRecord {
    /// Part 1: the radio's id (a network id or a radio class name).
    pub radio_id: String,
    /// Part 2: the frequencies the radio listens on, split at `|`; the plugin files the record
    /// under each.
    pub frequencies: Vec<String>,
    /// Part 3: the nickname of the player who carries the radio.
    pub owner_nickname: String,
    /// Part 4: the radio's position relative to the local listener, or `None` for `[]`, which the
    /// game sends for a radio its owner carries.
    pub position: Option<Position>,
    /// Part 5: the radio's volume, rounded.
    pub volume: i32,
    /// Part 6: the vehicle the radio is in.
    pub vehicle: VehicleDescriptor,
    /// Part 7: the height of the radio above sea level, which decides whether it is under water;
    /// 1 when the record has no seventh part.
    pub wave_z: f64,
}

/// `SPEAKERS\t<records>`: 1 field or more; the records are read only from a message of exactly 2.
pub(crate) fn parse_speakers(fields: &[&str]) -> Result<GameCommand> {
    let [_, records] = fields else {
        return Ok(GameCommand::Speakers(Vec::new()));
    };
    let mut speakers = Vec::new();
    for (index, record) in split_fields(records, SPEAKER_RECORD)
        .into_iter()
        .enumerate()
    {
        if !record.is_empty() {
            speakers.push(parse_speaker_record(index, record)?);
        }
    }
    Ok(GameCommand::Speakers(speakers))
}

/// One record: `radio id`, `frequencies`, `owner`, `[x,y,z]` or `[]`, `volume`, `vehicle id`
/// and an optional `wave z`, separated by line feeds; parts after the seventh are ignored.
fn parse_speaker_record(index: usize, record: &str) -> Result<SpeakerRecord> {
    let parts = split_fields(record, SPEAKER_PART);
    let [
        radio_id,
        frequencies,
        owner_nickname,
        position,
        volume,
        vehicle_id,
        rest @ ..,
    ] = parts.as_slice()
    else {
        return Err(Error::ShortSpeakerRecord {
            record: index,
            parts: parts.len(),
        });
    };
    let malformed = |field, source| Error::Malformed {
        command: SPEAKERS,
        field,
        source,
    };
    // The original reads a position only from text longer than `[]`.
    let position = if position.len() > 2 {
        parse_optional_position(position).map_err(|source| malformed("position", source))?
    } else {
        None
    };
    Ok(SpeakerRecord {
        radio_id: (*radio_id).to_owned(),
        frequencies: split_fields(frequencies, FREQUENCY_PART)
            .into_iter()
            .map(str::to_owned)
            .collect(),
        owner_nickname: (*owner_nickname).to_owned(),
        position,
        volume: rounded_integer(SPEAKERS, "volume", volume)?,
        vehicle: VehicleDescriptor::parse(vehicle_id)
            .map_err(|source| malformed("vehicle_id", source))?,
        wave_z: match rest.first() {
            Some(wave_z) => number(SPEAKERS, "wave_z", wave_z)?,
            None => DEFAULT_WAVE_Z,
        },
    })
}
