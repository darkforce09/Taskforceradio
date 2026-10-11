//! `TowerRecord` and the radio tower messages `RadioTwrAdd` and `RadioTwrDel`.

use tfar_foundation::geometry::Position;

use crate::game_commands::error::Result;
use crate::game_commands::fields::{
    MessageFields, NetworkObjectId, parse_coordinates, split_keeping_every_field,
};

#[cfg(test)]
#[path = "tests/radio_towers.rs"]
mod tests;

/// Between the records of a radio tower message (`TF_new_line`).
const TOWER_RECORD: char = '\n';

/// Between the parts of one tower record.
const TOWER_PART: char = ';';

/// One radio tower of a `RadioTwrAdd` message: `netId;range;[x,y,z]`.
#[derive(Clone, Debug, PartialEq)]
pub struct TowerRecord {
    /// The tower's network id; the antenna manager ignores a null id.
    pub net_id: NetworkObjectId,
    /// The tower's range in metres.
    pub range: f64,
    /// The tower's position.
    pub position: Position,
}

/// The towers of a `RadioTwrAdd` message (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:
/// 418-430`): exactly 2 tokens; a record without exactly three parts is skipped, as the original
/// skips it; a malformed range or position rejects the message.
pub(crate) fn parse_tower_records(fields: &MessageFields<'_>) -> Result<Vec<TowerRecord>> {
    fields.require_count(2)?;
    let mut towers = Vec::new();
    for record in split_keeping_every_field(fields.text(1)?, TOWER_RECORD) {
        let parts = split_keeping_every_field(record, TOWER_PART);
        let [net_id, range, position] = parts.as_slice() else {
            continue;
        };
        let range = tfar_foundation::arma_values::parse_number(range)
            .map_err(|_| fields.malformed("tower range", range))?;
        let position = parse_coordinates(position)
            .and_then(|coordinates| Position::from_array(coordinates).ok())
            .ok_or_else(|| fields.malformed("tower position", position))?;
        towers.push(TowerRecord {
            net_id: NetworkObjectId::parse(net_id),
            range,
            position,
        });
    }
    Ok(towers)
}

/// The tower ids of a `RadioTwrDel` message (`CommandProcessor.cpp:432-438`): exactly 2 tokens,
/// one id per record, every record kept (an empty one is the null id, which the antenna manager
/// ignores).
pub(crate) fn parse_tower_removals(fields: &MessageFields<'_>) -> Result<Vec<NetworkObjectId>> {
    fields.require_count(2)?;
    Ok(split_keeping_every_field(fields.text(1)?, TOWER_RECORD)
        .into_iter()
        .map(NetworkObjectId::parse)
        .collect())
}
