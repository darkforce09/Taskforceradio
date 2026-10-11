//! `Direction`: a unit vector in Arma's frame, or the zero vector when there is no direction.
//!
//! **Role:** hold a normalised direction and give its compass heading and polar angle.
//!
//! **Position:** a module of `tfar_foundation::geometry`; builds on `glam` and `Angle`; used by
//! `Position::direction_to`, the view directions of the client state and the spatial audio crates.
//!
//! **Signals and state:** none; a `Copy` value type, usable from any thread.
//!
//! **Invariants:** a direction has length 1 or is exactly zero, never NaN; `view_angle` is
//! `atan2(x, y)` and `polar_angle` is `atan2(y, x)`, both in [-π, π].

use glam::DVec3;

use crate::geometry::angle::Angle;
use crate::geometry::error::{Error, Result};

#[cfg(test)]
#[path = "tests/direction.rs"]
mod tests;

/// A direction in Arma's frame (x east, y north, z up): a vector of length 1, or the zero vector
/// when the source vector had no length.
///
/// Every constructor normalises, so a direction never carries the length of the vector it came
/// from and never holds NaN.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Direction(DVec3);

impl Direction {
    /// The zero direction: no direction at all.
    pub const ZERO: Self = Self(DVec3::ZERO);

    /// The direction of `vector`, scaled to length 1.
    ///
    /// A zero vector stays zero, and so does a vector whose length is not finite (NaN, infinite
    /// or overflowing), so the result is always a unit vector or zero.
    pub fn from_vector(vector: DVec3) -> Self {
        Self(vector.normalize_or_zero())
    }

    /// The direction of the vector `[x, y, z]`, scaled to length 1, as
    /// [`Direction::from_vector`] does.
    ///
    /// # Errors
    ///
    /// [`Error::NonFinite`] when a coordinate is NaN or infinite.
    pub fn from_array(coordinates: [f64; 3]) -> Result<Self> {
        let vector = DVec3::from_array(coordinates);
        if vector.is_finite() {
            Ok(Self::from_vector(vector))
        } else {
            Err(Error::NonFinite { coordinates })
        }
    }

    /// The direction as a `glam` vector.
    pub const fn as_dvec3(self) -> DVec3 {
        self.0
    }

    /// The direction as `[x, y, z]`.
    pub fn to_array(self) -> [f64; 3] {
        self.0.to_array()
    }

    /// The east component.
    pub const fn x(self) -> f64 {
        self.0.x
    }

    /// The north component.
    pub const fn y(self) -> f64 {
        self.0.y
    }

    /// The up component.
    pub const fn z(self) -> f64 {
        self.0.z
    }

    /// Whether this is the zero direction.
    pub fn is_zero(self) -> bool {
        self.0 == DVec3::ZERO
    }

    /// The compass heading of the direction, `atan2(x, y)`: 0 is north, π/2 east, and the result
    /// lies in [-π, π].
    ///
    /// This is the view angle of `reference/tfar_1_0/ts/src/datatypes.cpp` (`toAngle`).
    pub fn view_angle(self) -> Angle {
        Angle::from_radians(self.0.x.atan2(self.0.y))
    }

    /// The mathematical polar angle of the direction in the horizontal plane, `atan2(y, x)`: 0 is
    /// east, π/2 north, and the result lies in [-π, π].
    ///
    /// This is `toPolarAngle` of `reference/tfar_1_0/ts/src/datatypes.cpp`.
    pub fn polar_angle(self) -> Angle {
        Angle::from_radians(self.0.y.atan2(self.0.x))
    }
}
