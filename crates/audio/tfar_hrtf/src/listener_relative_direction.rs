//! `ListenerRelativeDirection`: where a sound comes from, seen from the listener's head.
//!
//! **Role:** turns Arma world positions and the listener's compass heading into a unit direction
//! in the listener's frame (right, forward, up), and that into the sampling vector of the HRIR
//! sphere's axes.
//!
//! **Position:** a module of `tfar_hrtf`; built by the voice pipelines from the game snapshot each
//! frame and passed to [`crate::HrtfSource::process`].
//!
//! **Signals and state:** none; a `Copy` value type, usable from any thread.
//!
//! **Invariants:** the direction always has unit length; a source at the listener's own position
//! (or any non-finite input) is straight ahead.
//!
//! **Axes.** Arma's world is x east, y north, z up, and the view angle is the compass heading
//! (0 north, π/2 east, clockwise seen from above). The IRCAM spheres of mrDIMAS's
//! `hrir_sphere_builder` (from its "Fix coordinate system to RH Cartesian" commit on) are
//! right-handed with x to the listener's right, y up and z backwards: the builder places azimuth
//! `T` and elevation `P` at `(cos P · cos(T + 90°), sin P, −cos P · sin(T + 90°))`, so the frontal
//! measurement `T000 P000` lies at `(0, 0, −1)` and the left one `T090 P000` at `(−1, 0, 0)`. The
//! `hrtf` crate samples the sphere along the vector it is given, so the sampling vector is the
//! direction from the listener to the source in those axes: `(right, up, −forward)`. The tests in
//! `tests/listener_relative_direction.rs` and `tests/hrtf_source.rs` pin both halves.

use glam::DVec3;
use tfar_geometry::{Angle, Position};

#[cfg(test)]
#[path = "tests/listener_relative_direction.rs"]
mod tests;

/// A unit direction from the listener to a sound source, in the listener's frame: `right` to the
/// listener's right ear, `forward` along the view, `up` above the head.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListenerRelativeDirection {
    right: f64,
    forward: f64,
    up: f64,
}

impl ListenerRelativeDirection {
    /// A source straight in front of the listener.
    pub const STRAIGHT_AHEAD: Self = Self {
        right: 0.0,
        forward: 1.0,
        up: 0.0,
    };

    /// The direction from a listener at `listener_position`, facing the compass heading
    /// `listener_view_angle`, to a source at `source_position`, all in Arma's world axes.
    ///
    /// Only the heading turns the frame: the listener's head is upright, so a source above stays
    /// above whichever way the listener looks. A source at the listener's own position is
    /// [`ListenerRelativeDirection::STRAIGHT_AHEAD`].
    pub fn from_arma(
        listener_position: Position,
        listener_view_angle: Angle,
        source_position: Position,
    ) -> Self {
        let offset = source_position.as_dvec3() - listener_position.as_dvec3();
        let (sine, cosine) = listener_view_angle.radians().sin_cos();
        // The view's forward axis is (sin h, cos h, 0) and its right axis (cos h, -sin h, 0).
        let forward = offset.x * sine + offset.y * cosine;
        let right = offset.x * cosine - offset.y * sine;
        Self::from_listener_axes(right, forward, offset.z)
    }

    /// The direction of the vector (`right`, `forward`, `up`) in the listener's frame, scaled to
    /// unit length. A zero or non-finite vector is
    /// [`ListenerRelativeDirection::STRAIGHT_AHEAD`].
    pub fn from_listener_axes(right: f64, forward: f64, up: f64) -> Self {
        let unit = DVec3::new(right, forward, up).normalize_or_zero();
        if unit == DVec3::ZERO {
            Self::STRAIGHT_AHEAD
        } else {
            Self {
                right: unit.x,
                forward: unit.y,
                up: unit.z,
            }
        }
    }

    /// The component toward the listener's right ear, in [-1, 1].
    pub const fn right(self) -> f64 {
        self.right
    }

    /// The component along the listener's view, in [-1, 1].
    pub const fn forward(self) -> f64 {
        self.forward
    }

    /// The component above the listener's head, in [-1, 1].
    pub const fn up(self) -> f64 {
        self.up
    }

    /// The `hrtf` crate's sampling vector for this direction: `(right, up, −forward)` in the
    /// sphere's right-handed axes (x right, y up, z backwards).
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the hrtf crate samples in f32; a unit component loses only precision"
    )]
    pub(crate) fn sphere_sampling_vector(self) -> hrtf::Vec3 {
        hrtf::Vec3::new(self.right as f32, self.up as f32, -self.forward as f32)
    }
}
