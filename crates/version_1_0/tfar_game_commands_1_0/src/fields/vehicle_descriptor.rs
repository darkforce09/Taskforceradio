//! `VehicleDescriptor`: the vehicle field of POS and SPEAKERS.
//!
//! **Role:** read `name\x10isolation\x10slot[\x10velocity]` or `no` into the vehicle a player or a
//! speaker is in, as `clientData::setVehicleId` (`reference/tfar_1_0/ts/src/clientData.hpp:
//! 349-371`) and `helpers::getVehicleDescriptor` (`helpers.cpp:330-350`) read it.
//!
//! **Position:** a module of the crate's `fields`; built on `tfar_arma_values` and
//! `parse_coordinates`; used by the POS and SPEAKERS parsers and by the client state.
//!
//! **Signals and state:** none; a value type.
//!
//! **Invariants:** `no` is the not-in-vehicle descriptor (name `no`, isolation 0, slot -1, no
//! velocity); `turnout` isolation is 0; a descriptor with fewer than three parts or a malformed
//! number is rejected, never half read.

use tfar_arma_values::separators::VEHICLE_PART;
use tfar_arma_values::{parse_int, parse_number};

use crate::fields::coordinates::parse_coordinates;

#[cfg(test)]
#[path = "tests/vehicle_descriptor.rs"]
mod tests;

/// The vehicle a player or a speaker is in.
#[derive(Clone, Debug, PartialEq)]
pub struct VehicleDescriptor {
    /// The vehicle's network id as the game writes it (`2:390`), or `no` when not in a vehicle;
    /// two players hear each other through a vehicle only when the names are equal.
    pub name: String,
    /// How much the vehicle isolates its crew from outside sound, 0 when turned out.
    pub isolation: f64,
    /// The intercom slot, -1 when the vehicle has no intercom.
    pub intercom_slot: i16,
    /// The unit's velocity in metres per second, `[x, y, z]`; zero when not sent.
    pub velocity: [f64; 3],
}

impl VehicleDescriptor {
    /// The vehicle name of a unit that is in no vehicle.
    pub const NOT_IN_VEHICLE_NAME: &'static str = "no";

    /// The isolation text of a turned-out unit, read as isolation 0.
    pub const TURNED_OUT: &'static str = "turnout";

    /// The descriptor of a unit in no vehicle: name `no`, isolation 0, slot -1, no velocity.
    pub fn not_in_vehicle() -> Self {
        Self {
            name: Self::NOT_IN_VEHICLE_NAME.to_owned(),
            isolation: 0.0,
            intercom_slot: -1,
            velocity: [0.0; 3],
        }
    }

    /// Whether the unit is in a vehicle (its name is not `no`).
    pub fn is_in_vehicle(&self) -> bool {
        self.name != Self::NOT_IN_VEHICLE_NAME
    }

    /// The descriptor in `text`, or `None` when it has fewer than three parts, an isolation or
    /// slot that is not a number, a slot outside the 16-bit range, or a malformed velocity.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        if text == Self::NOT_IN_VEHICLE_NAME {
            return Some(Self::not_in_vehicle());
        }
        let parts: Vec<&str> = text.split(VEHICLE_PART).collect();
        let (name, isolation, slot, velocity) = match parts.as_slice() {
            [name, isolation, slot] => (name, isolation, slot, None),
            [name, isolation, slot, velocity, ..] => (name, isolation, slot, Some(velocity)),
            _ => return None,
        };
        let isolation = if *isolation == Self::TURNED_OUT {
            0.0
        } else {
            parse_number(isolation).ok()?
        };
        let intercom_slot = i16::try_from(parse_int(slot).ok()?).ok()?;
        let velocity = match velocity {
            Some(velocity) => parse_coordinates(velocity)?,
            None => [0.0; 3],
        };
        Some(Self {
            name: (*name).to_owned(),
            isolation,
            intercom_slot,
            velocity,
        })
    }
}
