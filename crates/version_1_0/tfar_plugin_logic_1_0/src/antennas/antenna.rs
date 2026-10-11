//! `Antenna`: one radio tower, its position and its range.
//!
//! **Role:** hold one tower the game reports and measure the reach and loss of a hop through it,
//! as `Antenna` (`reference/tfar_1_0/ts/src/antennaManager.h:9-92`).
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::antennas`; built on `tfar_foundation::geometry::Position`; stored by
//! [`crate::antennas::AntennaManager`].
//!
//! **Signals and state:** none; a `Copy` value type.
//!
//! **Invariants:** the range is finite and not negative; reach is strictly inside a range; a
//! hop out of reach, or over a range of 0, costs a loss of 1.

use tfar_foundation::geometry::Position;

use crate::antennas::error::{Error, Result};
use crate::antennas::net_id::NetId;

#[cfg(test)]
#[path = "tests/antenna.rs"]
mod tests;

/// One radio tower the game reports: a relay that carries a transmission past the sender's own
/// range, as `Antenna` in `reference/tfar_1_0/ts/src/antennaManager.h:9-92`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Antenna {
    id: NetId,
    position: Position,
    range: f64,
}

impl Antenna {
    /// The tower `id` at `position` reaching `range` metres.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidRange`] when `range` is negative or not finite; the original accepts such
    /// a tower but can never connect through it.
    pub fn new(id: NetId, position: Position, range: f64) -> Result<Self> {
        if range.is_finite() && range >= 0.0 {
            Ok(Self {
                id,
                position,
                range,
            })
        } else {
            Err(Error::InvalidRange { id, range })
        }
    }

    /// The tower's network id.
    pub const fn id(&self) -> NetId {
        self.id
    }

    /// The tower's position.
    pub const fn position(&self) -> Position {
        self.position
    }

    /// How far the tower transmits, in metres.
    pub const fn range(&self) -> f64 {
        self.range
    }

    /// Whether a sender at `sender` with a radio of `sender_range` metres reaches the tower:
    /// strictly closer than its range (`canBeReachedBy`).
    pub fn is_reached_by(&self, sender: Position, sender_range: f64) -> bool {
        squared_distance(self.position, sender) < sender_range * sender_range
    }

    /// Whether the tower reaches a receiver at `receiver`: strictly closer than the tower's range
    /// (`canReach`).
    pub fn reaches(&self, receiver: Position) -> bool {
        squared_distance(self.position, receiver) < self.range * self.range
    }

    /// The signal loss of the hop from a sender at `sender` with `sender_range` metres to the
    /// tower and on to `receiver`: `d(receiver)/range + d(sender)/sender_range`, each part 1 when
    /// out of reach or when its range is 0 (`connectionLoss`, `lossTo`, `lossFrom`).
    pub fn connection_loss(&self, sender: Position, sender_range: f64, receiver: Position) -> f64 {
        hop_loss(self.position.distance_to(receiver), self.range)
            + hop_loss(self.position.distance_to(sender), sender_range)
    }
}

/// One hop's loss: the distance over the range, or 1 when the range is 0 or too short.
fn hop_loss(distance: f64, range: f64) -> f64 {
    if range <= 0.0 || distance > range {
        1.0
    } else {
        distance / range
    }
}

/// The squared distance between two positions, as `distanceToSqr` compares it.
fn squared_distance(from: Position, to: Position) -> f64 {
    from.as_dvec3().distance_squared(to.as_dvec3())
}
