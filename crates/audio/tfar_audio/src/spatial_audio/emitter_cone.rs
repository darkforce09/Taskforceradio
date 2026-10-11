//! `EmitterCone`: the speaker-direction gain that replaces the X3DAudio emitter cone.
//!
//! **Role:** gives the volume factor for a listener at some angle off the speaker's facing, with
//! X3DAudio's cone semantics, and the voice cone of the 1.0 plugin.
//!
//! **Position:** a module of `tfar_audio::spatial_audio`; used by the 1.0 voice
//! pipeline for direct speech when the `voiceCone` setting is on. The specification is the cone of
//! `reference/tfar_1_0/ts/src/helpers.cpp:90-93` and the cone evaluation of FAudio's `F3DAudio.c`
//! (`ComputeConeParameter`).
//!
//! **Signals and state:** none; a `Copy` value type, usable from any thread, nothing allocates.
//!
//! **Invariants:** `0 <= inner angle <= outer angle <= 2π` and both volumes lie within 0 to 2, as
//! X3DAudio requires; `gain` always returns a value between the inner and outer volume.

use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, TAU};

use tfar_foundation::geometry::{Angle, Direction};

use crate::spatial_audio::error::{Error, Result};

#[cfg(test)]
#[path = "tests/emitter_cone.rs"]
mod tests;

/// The largest cone volume X3DAudio accepts.
const MAX_CONE_VOLUME: f64 = 2.0;

/// A sound cone around an emitter's facing: the inner volume applies within half the inner angle
/// of the facing, the outer volume beyond half the outer angle, and the volume is interpolated
/// linearly between the two.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmitterCone {
    inner_angle: Angle,
    outer_angle: Angle,
    inner_volume: f64,
    outer_volume: f64,
}

impl EmitterCone {
    /// The voice cone of the 1.0 plugin: inner angle 90°, outer angle 135°, inner volume 1.8 and
    /// outer volume 1.0 (`reference/tfar_1_0/ts/src/helpers.cpp:90-93`), so a speaker facing the
    /// listener is 1.8 times as loud as one facing away.
    pub const TFAR_VOICE: Self = Self {
        inner_angle: Angle::from_radians(FRAC_PI_2),
        outer_angle: Angle::from_radians(3.0 * FRAC_PI_4),
        inner_volume: 1.8,
        outer_volume: 1.0,
    };

    /// A cone with full angles `inner_angle` and `outer_angle` and the volumes inside and outside.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidConeAngles`] unless `0 <= inner_angle <= outer_angle <= 2π`, and
    /// [`Error::InvalidConeVolume`] for a volume that is not finite and within 0 to 2.
    pub fn new(
        inner_angle: Angle,
        outer_angle: Angle,
        inner_volume: f64,
        outer_volume: f64,
    ) -> Result<Self> {
        let (inner, outer) = (inner_angle.radians(), outer_angle.radians());
        if !(inner >= 0.0 && inner <= outer && outer <= TAU) {
            return Err(Error::InvalidConeAngles {
                inner_angle: inner,
                outer_angle: outer,
            });
        }
        check_volume("inner", inner_volume)?;
        check_volume("outer", outer_volume)?;
        Ok(Self {
            inner_angle,
            outer_angle,
            inner_volume,
            outer_volume,
        })
    }

    /// The full inner angle.
    pub const fn inner_angle(self) -> Angle {
        self.inner_angle
    }

    /// The full outer angle.
    pub const fn outer_angle(self) -> Angle {
        self.outer_angle
    }

    /// The volume inside half the inner angle.
    pub const fn inner_volume(self) -> f64 {
        self.inner_volume
    }

    /// The volume beyond half the outer angle.
    pub const fn outer_volume(self) -> f64 {
        self.outer_volume
    }

    /// The volume factor for a listener in the direction `emitter_to_listener` from an emitter
    /// facing `emitter_front`.
    ///
    /// The angle between the two directions is `acos(front · to_listener)`. Within half the inner
    /// angle the inner volume applies, beyond half the outer angle the outer volume, and in
    /// between the volume is interpolated linearly, as FAudio's `ComputeConeParameter` does. A
    /// zero `emitter_to_listener` (the listener at the emitter) gets the inner volume; a zero
    /// `emitter_front` makes every angle 90°.
    pub fn gain(self, emitter_front: Direction, emitter_to_listener: Direction) -> f64 {
        if emitter_to_listener.is_zero() {
            return self.inner_volume;
        }
        let cosine = emitter_front
            .as_dvec3()
            .dot(emitter_to_listener.as_dvec3())
            .clamp(-1.0, 1.0);
        let angle = cosine.acos();
        let half_inner = self.inner_angle.radians() / 2.0;
        let half_outer = self.outer_angle.radians() / 2.0;
        if angle <= half_inner {
            self.inner_volume
        } else if angle <= half_outer {
            let position = (angle - half_inner) / (half_outer - half_inner);
            (1.0 - position) * self.inner_volume + position * self.outer_volume
        } else {
            self.outer_volume
        }
    }
}

/// Refuses a cone volume that is not finite and within 0 to 2.
fn check_volume(which: &'static str, volume: f64) -> Result<()> {
    if volume.is_finite() && (0.0..=MAX_CONE_VOLUME).contains(&volume) {
        Ok(())
    } else {
        Err(Error::InvalidConeVolume { which, volume })
    }
}
