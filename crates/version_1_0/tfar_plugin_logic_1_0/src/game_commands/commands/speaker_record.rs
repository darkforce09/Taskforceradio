//! `SpeakerRecord`: one radio playing over a loudspeaker, from the `SPEAKERS` message.
//!
//! **Role:** read the one field of `SPEAKERS` into its records, skipping a record with too few
//! parts instead of dropping the rest of the list.
//!
//! **Position:** a module of the crate's `commands`; built on `MessageFields`, the speaker
//! position reader and `VehicleDescriptor`; called by the parser.
//!
//! **Signals and state:** none; pure parsing.
//!
//! **Invariants:** records are split at `0x0B` and their parts at `0x0A`, every part kept, as
//! `processSpeakers` splits them (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:493-529`); an
//! empty record is skipped; a message of any token count but 2 is the empty list, which clears
//! the speakers as the original does; a record of fewer than six parts is skipped where the
//! original abandons the rest of the list.

use tfar_foundation::arma_values::separators::{FREQUENCY_PART, SPEAKER_PART, SPEAKER_RECORD};
use tfar_foundation::arma_values::{parse_int, parse_number, sanitize_nickname};
use tfar_foundation::geometry::Position;

use crate::game_commands::error::Result;
use crate::game_commands::fields::{MessageFields, VehicleDescriptor, parse_speaker_position};

#[cfg(test)]
#[path = "tests/speaker_record.rs"]
mod tests;

/// One radio playing over a loudspeaker.
#[derive(Clone, Debug, PartialEq)]
pub struct SpeakerRecord {
    /// The radio's id: a network id for a long-range radio, a class name for a short-range one.
    pub radio_id: String,
    /// The frequencies the radio plays, split at `|`, every part kept.
    pub frequencies: Vec<String>,
    /// The nickname of the radio's owner, sanitized as the plugin compares it.
    pub nickname: String,
    /// Where the speaker is; `None` when it is on its owner (sent as `[]` or as the zero
    /// position), whose position the plugin uses instead.
    pub position: Option<Position>,
    /// The radio volume, rounded.
    pub volume: i32,
    /// The vehicle the speaker is in.
    pub vehicle: VehicleDescriptor,
    /// The height that decides whether the speaker is under water: the seventh part when sent,
    /// else the position's height, else 1.
    pub wave_z: f64,
}

/// The fewest parts a speaker record has: radio id, frequencies, nickname, position, volume and
/// vehicle; the seventh, the wave height, is optional.
const MINIMUM_SPEAKER_PARTS: usize = 6;

/// The `SPEAKERS` message whose tokens are `fields`.
pub(crate) fn parse_speaker_records(fields: &MessageFields<'_>) -> Result<Vec<SpeakerRecord>> {
    let [list] = fields.after_name() else {
        return Ok(Vec::new());
    };
    let mut records = Vec::new();
    for record in list.split(SPEAKER_RECORD) {
        if record.is_empty() {
            continue;
        }
        let parts: Vec<&str> = record.split(SPEAKER_PART).collect();
        if parts.len() < MINIMUM_SPEAKER_PARTS {
            continue;
        }
        records.push(parse_record(fields, &parts)?);
    }
    Ok(records)
}

/// One record of at least six `parts`.
fn parse_record(fields: &MessageFields<'_>, parts: &[&str]) -> Result<SpeakerRecord> {
    let part = |index: usize| parts.get(index).copied().unwrap_or_default();
    let position = parse_speaker_position(part(3))
        .map_err(|()| fields.malformed("speaker position", part(3)))?;
    let volume = parse_int(part(4)).map_err(|_| fields.malformed("speaker volume", part(4)))?;
    let vehicle = VehicleDescriptor::parse(part(5))
        .ok_or_else(|| fields.malformed("speaker vehicle", part(5)))?;
    let wave_z = match parts.get(6) {
        Some(text) => {
            parse_number(text).map_err(|_| fields.malformed("speaker wave height", text))?
        }
        None => position.map_or(1.0, Position::z),
    };
    Ok(SpeakerRecord {
        radio_id: part(0).to_owned(),
        frequencies: part(1).split(FREQUENCY_PART).map(str::to_owned).collect(),
        nickname: sanitize_nickname(part(2)),
        position,
        volume,
        vehicle,
        wave_z,
    })
}
