//! `POS`: one player's position relative to the local listener, and what that player can do.
//!
//! **Role:** read the 13 fields of a `POS` message into a `PositionUpdate`.
//!
//! **Position:** a message parser of `tfar_plugin_logic_0_9::game_commands`, called by `GameCommand::parse`;
//! follows the `POS` branch of `processGameCommand` in `reference/tfar_0_9_12/ts/src/plugin.cpp`.
//!
//! **Signals and state:** none; a pure function.
//!
//! **Invariants:** exactly 14 fields; the booleans are the exact text `true`; the terrain
//! interception truncates as `std::stoi` does; a malformed number or vehicle id is an error
//! naming its field, where the original's `std::stof` throws.

use tfar_foundation::geometry::{Angle, Position};

use crate::game_commands::command_names::POS;
use crate::game_commands::error::{Error, Result};
use crate::game_commands::field_values::{exact_fields, is_true, number, truncated_integer};
use crate::game_commands::game_command::GameCommand;
use crate::game_commands::vehicle_descriptor::VehicleDescriptor;

#[cfg(test)]
#[path = "tests/unit_position.rs"]
mod tests;

/// The 13 fields of a `POS` message, typed, in the order
/// `processGameCommand` in `reference/tfar_0_9_12/ts/src/plugin.cpp` reads them.
#[derive(Clone, Debug, PartialEq)]
pub struct PositionUpdate {
    /// Field 1: the player's nickname.
    pub nickname: String,
    /// Fields 2 to 4: the player's position in metres relative to the local player's eyes; the
    /// local player's own position is the origin.
    pub position: Position,
    /// Field 5: the direction the player looks, from degrees; the plugin keeps it for the local
    /// player only.
    pub view_angle: Angle,
    /// Field 6: whether the player can speak (not under water, not in an isolated vehicle).
    pub can_speak: bool,
    /// Fields 7 to 9: which radio families the player can use.
    pub usable_radios: UsableRadios,
    /// Field 10: the vehicle the player sits in.
    pub vehicle: VehicleDescriptor,
    /// Field 11: how much terrain lies between the player and the local listener, truncated to an
    /// integer as `std::stoi` truncates it.
    pub terrain_interception: i32,
    /// Field 12: the player's voice volume multiplier.
    pub voice_volume_multiplier: f64,
    /// Field 13: the direction the local player faces, from degrees.
    pub current_direction: Angle,
}

/// Which radio families a player can use, from fields 7 to 9 of `POS`: the game clears them
/// under water or in an isolated vehicle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UsableRadios {
    /// Field 7: a short-range radio.
    pub short_range: bool,
    /// Field 8: a long-range radio.
    pub long_range: bool,
    /// Field 9: a diver radio.
    pub diver: bool,
}

/// `POS` and 13 fields: exactly 14. The booleans are `"true"` or false; every number must parse.
pub(crate) fn parse_unit_position(fields: &[&str]) -> Result<GameCommand> {
    let [
        _,
        nickname,
        x,
        y,
        z,
        view_angle,
        can_speak,
        can_use_short_range_radio,
        can_use_long_range_radio,
        can_use_diver_radio,
        vehicle_id,
        terrain_interception,
        voice_volume_multiplier,
        current_direction,
    ] = exact_fields::<14>(POS, fields)?;
    let coordinates = [
        number(POS, "x", x)?,
        number(POS, "y", y)?,
        number(POS, "z", z)?,
    ];
    let position = Position::from_array(coordinates).map_err(|source| Error::Malformed {
        command: POS,
        field: "position",
        source: source.into(),
    })?;
    let vehicle = VehicleDescriptor::parse(vehicle_id).map_err(|source| Error::Malformed {
        command: POS,
        field: "vehicle_id",
        source,
    })?;
    Ok(GameCommand::Position(PositionUpdate {
        nickname: nickname.to_owned(),
        position,
        view_angle: Angle::from_degrees(number(POS, "view_angle", view_angle)?),
        can_speak: is_true(can_speak),
        usable_radios: UsableRadios {
            short_range: is_true(can_use_short_range_radio),
            long_range: is_true(can_use_long_range_radio),
            diver: is_true(can_use_diver_radio),
        },
        vehicle,
        terrain_interception: truncated_integer(POS, "terrain_interception", terrain_interception)?,
        voice_volume_multiplier: number(POS, "voice_volume_multiplier", voice_volume_multiplier)?,
        current_direction: Angle::from_degrees(number(
            POS,
            "current_direction",
            current_direction,
        )?),
    }))
}
