//! `rotate_about_z`: turning a position about the vertical axis.

use glam::DVec3;

use crate::geometry::angle::Angle;
use crate::geometry::position::Position;

#[cfg(test)]
#[path = "tests/rotation.rs"]
mod tests;

/// `position` turned counter-clockwise (seen from above) by `angle` about the z axis through the
/// origin: `x' = x cos θ − y sin θ`, `y' = x sin θ + y cos θ`, z unchanged.
///
/// This is the rotation of `reference/tfar_0_9_12/ts/src/Clunk.h`, which turns a speaker's
/// relative position into the listener's frame.
pub fn rotate_about_z(position: Position, angle: Angle) -> Position {
    let (sine, cosine) = angle.radians().sin_cos();
    let DVec3 { x, y, z } = position.as_dvec3();
    Position(DVec3::new(x * cosine - y * sine, x * sine + y * cosine, z))
}
