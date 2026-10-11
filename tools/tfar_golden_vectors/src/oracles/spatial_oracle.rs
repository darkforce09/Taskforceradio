//! The F3DAudio oracle: FAudio's X3DAudio reimplementation, run for one listener and one mono
//! emitter.
//!
//! **Role:** describes a scene (listener and emitter positions and orientations, an optional
//! volume curve and an optional emitter cone) and runs `F3DAudioCalculate` on it through the
//! shim, returning the stereo matrix, the volume it splits between the speakers, the emitter angle
//! and the X3DAudio parameter checks the scene fails.
//!
//! **Position:** inside `oracles`; called by the spatial comparison tests, which compare the volume
//! with `tfar_audio::spatial_audio`'s distance curve and emitter cone. The scene mirrors
//! `applyILD` in `reference/tfar_1_0/ts/src/helpers.cpp`: a stereo output, a one-channel emitter
//! with a channel radius of 1, the curve scaled by `CurveDistanceScaler`.
//!
//! **Signals and state:** none kept: each call initialises its own F3DAudio handle. Any thread may
//! call; the failed-check count is the calling thread's own.
//!
//! **Invariants:** every value crosses as `f32`, the precision X3DAudio computes in; the volume is
//! the sum of the two matrix coefficients, which F3DAudio's speaker panning splits without loss,
//! so it is the curve gain times the cone gain.

use crate::error::{Error, Result};
use crate::oracles::oracle_bindings::{self, SHIM_MAX_CURVE_POINTS, ShimSpatialScene, ShimVector};

#[cfg(test)]
#[path = "tests/spatial_oracle.rs"]
mod tests;

/// The most volume curve points a scene carries.
pub const ORACLE_MAX_CURVE_POINTS: usize = SHIM_MAX_CURVE_POINTS;

/// An X3DAudio emitter cone: full angles in radians, volumes as linear factors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OracleCone {
    /// The full inner angle.
    pub inner_angle: f32,
    /// The full outer angle.
    pub outer_angle: f32,
    /// The volume inside half the inner angle.
    pub inner_volume: f32,
    /// The volume beyond half the outer angle.
    pub outer_volume: f32,
}

/// One listener, one mono emitter, and how the emitter's volume falls off.
#[derive(Debug, Clone, PartialEq)]
pub struct OracleSpatialScene {
    /// The listener's position.
    pub listener_position: [f32; 3],
    /// The listener's facing, a unit vector.
    pub listener_front: [f32; 3],
    /// The listener's up, a unit vector orthogonal to the facing.
    pub listener_top: [f32; 3],
    /// The emitter's position.
    pub emitter_position: [f32; 3],
    /// The emitter's facing, a unit vector.
    pub emitter_front: [f32; 3],
    /// The emitter's up, a unit vector orthogonal to the facing.
    pub emitter_top: [f32; 3],
    /// The volume curve as `(normalised distance, gain)` points; empty for X3DAudio's default
    /// inverse-distance law.
    pub volume_curve: Vec<(f32, f32)>,
    /// `CurveDistanceScaler`: the distance in metres that normalised distance 1 stands for.
    pub curve_distance_scaler: f32,
    /// The emitter cone, or `None` for an omnidirectional emitter.
    pub cone: Option<OracleCone>,
}

/// What `F3DAudioCalculate` computed for a scene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OracleSpatialResult {
    /// The left speaker's matrix coefficient.
    pub left: f32,
    /// The right speaker's matrix coefficient.
    pub right: f32,
    /// `left + right`: the curve gain times the cone gain.
    pub volume: f32,
    /// The angle between the emitter's facing and the emitter-to-listener direction, in radians.
    pub emitter_to_listener_angle: f32,
    /// The emitter-to-listener distance.
    pub emitter_to_listener_distance: f32,
    /// How many X3DAudio parameter checks the scene failed (F3DAudio computes regardless).
    pub failed_parameter_checks: u32,
}

/// Runs `F3DAudioCalculate` for `scene` with the matrix and emitter-angle flags.
///
/// # Errors
///
/// [`Error::TooManyCurvePoints`] beyond [`ORACLE_MAX_CURVE_POINTS`];
/// [`Error::InvalidParameter`] for a distance scaler that is not finite and above 0;
/// [`Error::OracleRefused`] when the shim refuses the call.
pub fn f3daudio_calculate(scene: &OracleSpatialScene) -> Result<OracleSpatialResult> {
    let scaler = f64::from(scene.curve_distance_scaler);
    if !(scaler.is_finite() && scaler > 0.0) {
        return Err(Error::InvalidParameter {
            parameter: "curve_distance_scaler",
            value: scaler,
            requirement: "finite and above 0",
        });
    }
    let given = scene.volume_curve.len();
    let point_count = u32::try_from(given)
        .ok()
        .filter(|_| given <= ORACLE_MAX_CURVE_POINTS)
        .ok_or(Error::TooManyCurvePoints {
            given,
            maximum: ORACLE_MAX_CURVE_POINTS,
        })?;
    let mut shim_scene = ShimSpatialScene {
        listener_position: shim_vector(scene.listener_position),
        listener_front: shim_vector(scene.listener_front),
        listener_top: shim_vector(scene.listener_top),
        emitter_position: shim_vector(scene.emitter_position),
        emitter_front: shim_vector(scene.emitter_front),
        emitter_top: shim_vector(scene.emitter_top),
        curve_point_count: point_count,
        curve_distance_scaler: scene.curve_distance_scaler,
        ..ShimSpatialScene::default()
    };
    let slots = shim_scene
        .curve_distances
        .iter_mut()
        .zip(shim_scene.curve_gains.iter_mut());
    for ((distance_slot, gain_slot), (distance, gain)) in slots.zip(&scene.volume_curve) {
        *distance_slot = *distance;
        *gain_slot = *gain;
    }
    if let Some(cone) = scene.cone {
        shim_scene.has_cone = 1;
        shim_scene.cone_inner_angle = cone.inner_angle;
        shim_scene.cone_outer_angle = cone.outer_angle;
        shim_scene.cone_inner_volume = cone.inner_volume;
        shim_scene.cone_outer_volume = cone.outer_volume;
    }
    let result = oracle_bindings::f3daudio_calculate(&shim_scene)?;
    Ok(OracleSpatialResult {
        left: result.left,
        right: result.right,
        volume: result.volume,
        emitter_to_listener_angle: result.emitter_to_listener_angle,
        emitter_to_listener_distance: result.emitter_to_listener_distance,
        failed_parameter_checks: result.assertion_failures,
    })
}

/// A coordinate triple as the shim's vector.
fn shim_vector([x, y, z]: [f32; 3]) -> ShimVector {
    ShimVector { x, y, z }
}
