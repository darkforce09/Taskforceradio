//! `PositionUpdate`: the `POS` message, one player's position and state.
//!
//! **Role:** read the thirteen fields of `POS` into a typed update, rejecting a wrong token
//! count or a malformed field.
//!
//! **Position:** a module of the crate's `commands`; built on `MessageFields`, the coordinate
//! reader and `VehicleDescriptor`; called by the parser.
//!
//! **Signals and state:** none; pure parsing.
//!
//! **Invariants:** the fields are those of `unitPositionPacket`
//! (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:263-277`) in that order, the message is
//! exactly 14 tokens, and booleans read as `helpers::isTrue` reads them.

use tfar_foundation::geometry::{Direction, Position};

use crate::game_commands::error::Result;
use crate::game_commands::fields::{MessageFields, VehicleDescriptor, parse_coordinates};

#[cfg(test)]
#[path = "tests/position_update.rs"]
mod tests;

/// One `POS` message: where a player is and what they can do.
#[derive(Clone, Debug, PartialEq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is one independent field of the POS message"
)]
pub struct PositionUpdate {
    /// The player's nickname, sanitized as the plugin compares it.
    pub nickname: String,
    /// The player's eye position in world coordinates.
    pub position: Position,
    /// The player's view direction.
    pub view_direction: Direction,
    /// The player can speak (alive, or a spectator, and not under water).
    pub can_speak: bool,
    /// The player can use a short-range radio.
    pub can_use_sw_radio: bool,
    /// The player can use a long-range radio.
    pub can_use_lr_radio: bool,
    /// The player can use a diver radio.
    pub can_use_dd_radio: bool,
    /// The vehicle the player is in.
    pub vehicle: VehicleDescriptor,
    /// The terrain between the player and the local player, rounded.
    pub terrain_interception: i32,
    /// The player's voice volume multiplier.
    pub voice_volume: f64,
    /// The objects between the player and the local player, rounded.
    pub object_interception: i32,
    /// The player is a spectator.
    pub is_spectating: bool,
    /// The player is an enemy of the local player (sent only while the local player spectates).
    pub is_enemy_to_player: bool,
}

/// The `POS` message whose tokens are `fields`.
pub(crate) fn parse_position_update(fields: &MessageFields<'_>) -> Result<PositionUpdate> {
    fields.require_count(14)?;
    let position = fields.read(2, "position", |text| {
        parse_coordinates(text).and_then(|coordinates| Position::from_array(coordinates).ok())
    })?;
    let view_direction = fields.read(3, "view direction", |text| {
        parse_coordinates(text).and_then(|coordinates| Direction::from_array(coordinates).ok())
    })?;
    Ok(PositionUpdate {
        nickname: fields.nickname(1)?,
        position,
        view_direction,
        can_speak: fields.truth(4)?,
        can_use_sw_radio: fields.truth(5)?,
        can_use_lr_radio: fields.truth(6)?,
        can_use_dd_radio: fields.truth(7)?,
        vehicle: fields.read(8, "vehicle", VehicleDescriptor::parse)?,
        terrain_interception: fields.integer(9, "terrain interception")?,
        voice_volume: fields.number(10, "voice volume")?,
        object_interception: fields.integer(11, "object interception")?,
        is_spectating: fields.truth(12)?,
        is_enemy_to_player: fields.truth(13)?,
    })
}
