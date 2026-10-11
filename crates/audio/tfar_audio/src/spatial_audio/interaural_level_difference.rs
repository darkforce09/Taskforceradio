//! The simple interaural level difference: left and right gains from the direction of a sound.
//!
//! **Role:** turns a sound's direction relative to the listener and the listener's view angle into
//! a left and a right gain, with the coefficient of the 0.9 or the 1.0 plugin, and applies them.
//!
//! **Position:** a module of `tfar_audio::spatial_audio`; the fallback panner of
//! the voice pipelines when the HRIR sphere cannot be loaded, and the panner of plugin-folder
//! playbacks. The specification is `applyILD` in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:608-635` and
//! `reference/tfar_1_0/ts/src/helpers.cpp:12-22`, with the angle wrapped in both directions.
//!
//! **Signals and state:** none; pure functions over caller buffers, callable from the TeamSpeak
//! audio thread; nothing allocates.
//!
//! **Invariants:** with `dir = (polar angle + view angle)` wrapped into (-π, π], the gains are
//! `left = -c * cos(dir) + 0.625` and `right = c * cos(dir) + 0.625`; equal when `dir = ±π/2`.

use crate::sample_buffer::{StereoBuffer, apply_stereo_gain};
use tfar_foundation::geometry::{Angle, Direction};

#[cfg(test)]
#[path = "tests/interaural_level_difference.rs"]
mod tests;

/// The ILD coefficient of the 0.9 plugin: the literal 0.375 of
/// `reference/tfar_0_9_12/ts/src/plugin.cpp:621-622` (`gainLeft = -0.375 * cos(dir) + 0.625`).
pub const ILD_COEFFICIENT_0_9: f32 = 0.375;

/// The ILD coefficient of the 1.0 plugin: 21.5° in radians, computed in `f32` as
/// `reference/tfar_1_0/ts/src/helpers.cpp:17-18` does (`AngleDegrees(21.5f).toRadians()`, which is
/// `21.5f * (M_PI_FLOAT / 180)` by `reference/tfar_1_0/ts/src/datatypes.hpp:81`), about 0.375246.
pub const ILD_COEFFICIENT_1_0: f32 = 21.5 * (core::f32::consts::PI / 180.0);

/// The gain both ears share before the direction term, 0.625 in both originals.
pub const ILD_BASE_GAIN: f32 = 0.625;

/// The `(left, right)` gains for a sound in `relative_direction` from the listener, who looks at
/// `view_angle`, with the version's `coefficient` ([`ILD_COEFFICIENT_0_9`] or
/// [`ILD_COEFFICIENT_1_0`]).
///
/// `dir = relative_direction.polar_angle() + view_angle`, wrapped into (-π, π] by
/// [`Angle::normalized_signed`]; then `left = -coefficient * cos(dir) + 0.625` and
/// `right = coefficient * cos(dir) + 0.625`. A sound at `dir = 0` is on the right, at `dir = π` on
/// the left, and at `dir = ±π/2` both gains are 0.625.
pub fn stereo_pan_gains(
    relative_direction: Direction,
    view_angle: Angle,
    coefficient: f32,
) -> (f32, f32) {
    let direction_angle = (relative_direction.polar_angle() + view_angle).normalized_signed();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the gains are f32 as in both originals; a cosine lies in [-1, 1]"
    )]
    let cosine = direction_angle.cos() as f32;
    let left = -coefficient * cosine + ILD_BASE_GAIN;
    let right = coefficient * cosine + ILD_BASE_GAIN;
    (left, right)
}

/// Multiplies the left channel of `stereo` by `gains.0` and the right channel by `gains.1`,
/// saturating at the `i16` limits.
pub fn apply_pan(stereo: &mut StereoBuffer, gains: (f32, f32)) {
    let (left, right) = gains;
    apply_stereo_gain(stereo, left, right);
}
