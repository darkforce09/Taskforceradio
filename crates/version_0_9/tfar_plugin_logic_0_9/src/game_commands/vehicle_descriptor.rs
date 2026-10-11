//! `VehicleDescriptor`: the vehicle a player sits in and how much of the outside it shuts out.

use tfar_foundation::arma_values::parse_number;

#[cfg(test)]
#[path = "tests/vehicle_descriptor.rs"]
mod tests;

/// The vehicle id `POS` and `SPEAKERS` carry, read as `getVehicleDescriptor` in
/// `reference/tfar_0_9_12/ts/src/plugin.cpp` reads it. The game writes `no` on foot,
/// `<netId>_turnout` for a turned-out crew member and `<netId>_<isolation>` inside.
#[derive(Clone, Debug, PartialEq)]
pub struct VehicleDescriptor {
    /// The vehicle the id names (`2:345`, `singleplayer`), or the whole id when it has no `_`
    /// (`no`); two players share a vehicle when their names are equal.
    pub vehicle_name: String,
    /// The vehicle's `tf_isolatedAmount` from 0 to 1; 0 when turned out or not in a vehicle.
    pub isolation: f64,
}

impl VehicleDescriptor {
    /// The descriptor of `vehicle_id`: the text before the first `_turnout` with no isolation;
    /// else the text before the last `_` with the number after it as isolation; else the whole id
    /// with no isolation.
    ///
    /// # Errors
    ///
    /// [`tfar_foundation::arma_values::Error::MalformedNumber`] when the text after the last `_` is not a
    /// number.
    pub fn parse(vehicle_id: &str) -> tfar_foundation::arma_values::Result<Self> {
        if let Some((vehicle_name, _)) = vehicle_id.split_once("_turnout") {
            return Ok(Self {
                vehicle_name: vehicle_name.to_owned(),
                isolation: 0.0,
            });
        }
        match vehicle_id.rsplit_once('_') {
            Some((vehicle_name, isolation)) => Ok(Self {
                vehicle_name: vehicle_name.to_owned(),
                isolation: parse_number(isolation)?,
            }),
            None => Ok(Self {
                vehicle_name: vehicle_id.to_owned(),
                isolation: 0.0,
            }),
        }
    }
}
