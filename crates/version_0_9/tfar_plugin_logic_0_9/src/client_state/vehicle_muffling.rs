//! `VehicleMuffling`: how much a vehicle between a sound and the listener shuts it out.

use crate::game_commands::VehicleDescriptor;

#[cfg(test)]
#[path = "tests/vehicle_muffling.rs"]
mod tests;

/// The highest combined isolation: a sound is never shut out completely.
pub const MAX_COMBINED_ISOLATION: f64 = 0.99;

/// The combined isolation below which a sound is not muffled.
pub const MUFFLING_THRESHOLD: f64 = 0.01;

/// The muffling between the listener's vehicle and a sound's vehicle, as the stereo function and
/// the positional sounds compute it in `reference/tfar_0_9_12/ts/src/plugin.cpp:2691-2695`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleMuffling {
    /// The sum of both isolations, clamped to `[0, MAX_COMBINED_ISOLATION]`.
    pub isolation: f64,
    /// Whether both sit in the same vehicle (their vehicle names are equal).
    pub is_same_vehicle: bool,
}

impl VehicleMuffling {
    /// The muffling between `listener` and `source`.
    pub fn between(listener: &VehicleDescriptor, source: &VehicleDescriptor) -> Self {
        Self {
            isolation: (listener.isolation + source.isolation).clamp(0.0, MAX_COMBINED_ISOLATION),
            is_same_vehicle: listener.vehicle_name == source.vehicle_name,
        }
    }

    /// Whether the sound goes through the vehicle low-pass: the combined isolation reaches
    /// [`MUFFLING_THRESHOLD`] and the two are not in the same vehicle.
    pub fn muffles(self) -> bool {
        !(self.isolation < MUFFLING_THRESHOLD || self.is_same_vehicle)
    }

    /// The combined isolation when the sound is muffled, else `None`.
    pub fn muffling_isolation(self) -> Option<f64> {
        self.muffles().then_some(self.isolation)
    }
}
