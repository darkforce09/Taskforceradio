//! `Position`: a point in Arma's frame, world or relative, in metres.
//!
//! **Role:** hold a finite point and measure from it: the distance to another point and the unit
//! direction towards it.
//!
//! **Position:** a module of `tfar_foundation::geometry`; builds on `glam` and `Direction`; used by the
//! underwater distance, the rotation and every positional crate above.
//!
//! **Signals and state:** none; a `Copy` value type, usable from any thread.
//!
//! **Invariants:** the constructors accept only finite coordinates; `direction_to` returns a unit
//! vector, or zero for two equal positions.

use glam::DVec3;

use crate::geometry::direction::Direction;
use crate::geometry::error::{Error, Result};

#[cfg(test)]
#[path = "tests/position.rs"]
mod tests;

/// A point in Arma's frame (x east, y north, z up), in metres: a world position (`getPosASL`
/// style, z = 0 at sea level) or a position relative to a listener.
///
/// The constructors accept only finite coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(pub(crate) DVec3);

impl Position {
    /// The origin `[0, 0, 0]`.
    pub const ORIGIN: Self = Self(DVec3::ZERO);

    /// The position `[x, y, z]`.
    ///
    /// # Errors
    ///
    /// [`Error::NonFinite`] when a coordinate is NaN or infinite.
    pub fn from_array(coordinates: [f64; 3]) -> Result<Self> {
        Self::from_dvec3(DVec3::from_array(coordinates))
    }

    /// The position held by a `glam` vector.
    ///
    /// # Errors
    ///
    /// [`Error::NonFinite`] when a coordinate is NaN or infinite.
    pub fn from_dvec3(vector: DVec3) -> Result<Self> {
        if vector.is_finite() {
            Ok(Self(vector))
        } else {
            Err(Error::NonFinite {
                coordinates: vector.to_array(),
            })
        }
    }

    /// The position as a `glam` vector.
    pub const fn as_dvec3(self) -> DVec3 {
        self.0
    }

    /// The position as `[x, y, z]`.
    pub fn to_array(self) -> [f64; 3] {
        self.0.to_array()
    }

    /// The east coordinate, in metres.
    pub const fn x(self) -> f64 {
        self.0.x
    }

    /// The north coordinate, in metres.
    pub const fn y(self) -> f64 {
        self.0.y
    }

    /// The height, in metres; below 0 is under the sea surface.
    pub const fn z(self) -> f64 {
        self.0.z
    }

    /// The straight-line distance to `other`, in metres.
    pub fn distance_to(self, other: Self) -> f64 {
        self.0.distance(other.0)
    }

    /// The unit direction from this position towards `other`; zero when the two coincide.
    pub fn direction_to(self, other: Self) -> Direction {
        Direction::from_vector(other.0 - self.0)
    }
}
