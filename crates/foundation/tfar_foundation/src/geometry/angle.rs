//! `Angle`: an angle in radians, with the degree conversions and the signed wrap the spatial
//! audio code needs.
//!
//! **Role:** carry angles as radians in one type, so degrees and radians never mix unnoticed.
//!
//! **Position:** a module of `tfar_foundation::geometry`; returned by `Direction`'s angles and taken
//! by `rotate_about_z`; used by the interaural level difference and the HRTF direction code.
//!
//! **Signals and state:** none; a `Copy` value type, usable from any thread.
//!
//! **Invariants:** `normalized_signed` returns a value in (-π, π] for every finite input.

use core::f64::consts::{PI, TAU};
use core::ops::{Add, Neg, Sub};

#[cfg(test)]
#[path = "tests/angle.rs"]
mod tests;

/// An angle in radians.
///
/// The value is whatever was given or computed; it is not wrapped until
/// [`Angle::normalized_signed`] is called. A non-finite input stays non-finite.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Angle(f64);

impl Angle {
    /// The zero angle.
    pub const ZERO: Self = Self(0.0);

    /// An angle of `radians` radians.
    pub const fn from_radians(radians: f64) -> Self {
        Self(radians)
    }

    /// An angle of `degrees` degrees.
    pub fn from_degrees(degrees: f64) -> Self {
        Self(degrees.to_radians())
    }

    /// The angle in radians.
    pub const fn radians(self) -> f64 {
        self.0
    }

    /// The angle in degrees.
    pub fn to_degrees(self) -> f64 {
        self.0.to_degrees()
    }

    /// The cosine of the angle.
    pub fn cos(self) -> f64 {
        self.0.cos()
    }

    /// The sine of the angle.
    pub fn sin(self) -> f64 {
        self.0.sin()
    }

    /// The same angle wrapped into (-π, π]: any multiple of 2π is removed, in both directions.
    ///
    /// The interaural level difference adds a polar angle and a view angle, whose sum can lie
    /// anywhere in (-2π, 2π]; the wrap brings it back so `-π` and `π` both map to `π`.
    #[must_use]
    pub fn normalized_signed(self) -> Self {
        let wrapped = self.0.rem_euclid(TAU);
        if wrapped > PI {
            Self(wrapped - TAU)
        } else {
            Self(wrapped)
        }
    }
}

impl Add for Angle {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for Angle {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}
