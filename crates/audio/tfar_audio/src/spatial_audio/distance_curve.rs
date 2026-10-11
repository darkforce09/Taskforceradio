//! `DistanceCurve`: the piecewise-linear gain by normalised distance that replaces the X3DAudio
//! volume curve.
//!
//! **Role:** holds sorted `(normalised distance, gain)` points and reads the gain at a distance
//! the way X3DAudio's volume curve does; builds the 25-point voice curve of the 1.0 plugin.
//!
//! **Position:** a module of `tfar_audio::spatial_audio`; used by the voice
//! pipelines for the level of direct speech. The specification is the volume curve of
//! `reference/tfar_1_0/ts/src/helpers.cpp:113-140` and the curve evaluation of FAudio's
//! `F3DAudio.c` (`ComputeCurveValue`), the oracle of the golden vectors.
//!
//! **Signals and state:** none; a `Copy` value type held in a fixed array, so building a curve and
//! reading it never allocate and may run on the TeamSpeak audio thread.
//!
//! **Invariants:** a curve holds 1 to [`DistanceCurve::MAX_POINTS`] points with finite,
//! non-negative distances in strictly ascending order and finite, non-negative gains; `gain_at`
//! always returns one of those gains or a value between two neighbouring ones.

use crate::spatial_audio::error::{Error, Result};

#[cfg(test)]
#[path = "tests/distance_curve.rs"]
mod tests;

/// One point of a [`DistanceCurve`]: the gain at a normalised distance.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CurvePoint {
    /// The distance divided by the emitter's distance scaler.
    pub distance: f64,
    /// The gain at that distance.
    pub gain: f64,
}

/// A piecewise-linear gain curve over normalised distance, with X3DAudio's semantics: the
/// distance is divided by the emitter's scaler, the gain between two points is interpolated
/// linearly, the first gain holds before the first point and the last gain holds past the end.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DistanceCurve {
    points: [CurvePoint; DistanceCurve::MAX_POINTS],
    point_count: usize,
}

/// The normalised distances of the 1.0 voice curve,
/// `reference/tfar_1_0/ts/src/helpers.cpp:114-138`.
const TFAR_VOICE_DISTANCES: [f64; 25] = [
    0.0, 0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40, 0.45, 0.50, 0.55, 0.60, 0.65, 0.70, 0.75,
    0.80, 0.85, 0.90, 0.95, 1.05, 1.10, 1.15, 1.20, 1.25,
];

/// For each point of the 1.0 voice curve, the multiple of `voice_volume / 20` metres its gain is
/// the attenuation of; `None` for the fixed first point (gain 1) and last point (gain 0).
const TFAR_VOICE_METRE_STEPS: [Option<f64>; 25] = [
    None,
    Some(1.0),
    Some(2.0),
    Some(3.0),
    Some(4.0),
    Some(5.0),
    Some(6.0),
    Some(7.0),
    Some(8.0),
    Some(9.0),
    Some(10.0),
    Some(11.0),
    Some(12.0),
    Some(13.0),
    Some(14.0),
    Some(15.0),
    Some(16.0),
    Some(17.0),
    Some(18.0),
    Some(19.0),
    Some(20.0),
    Some(24.0),
    Some(28.0),
    Some(32.0),
    None,
];

impl DistanceCurve {
    /// The most points a curve holds; the 1.0 voice curve uses 25.
    pub const MAX_POINTS: usize = 32;

    /// A curve through `points`, given as `(normalised_distance, gain)` in ascending distance.
    ///
    /// # Errors
    ///
    /// [`Error::EmptyDistanceCurve`] for no point, [`Error::TooManyCurvePoints`] beyond
    /// [`DistanceCurve::MAX_POINTS`], [`Error::InvalidCurveDistance`] or
    /// [`Error::InvalidCurveGain`] for a value that is not finite and non-negative, and
    /// [`Error::UnsortedCurvePoints`] when a distance is not greater than the one before it.
    pub fn new(points: impl IntoIterator<Item = (f64, f64)>) -> Result<Self> {
        let mut curve = Self {
            points: [CurvePoint::default(); Self::MAX_POINTS],
            point_count: 0,
        };
        let mut given = 0_usize;
        for (index, (distance, gain)) in points.into_iter().enumerate() {
            given = index.saturating_add(1);
            if !(distance.is_finite() && distance >= 0.0) {
                return Err(Error::InvalidCurveDistance { index, distance });
            }
            if !(gain.is_finite() && gain >= 0.0) {
                return Err(Error::InvalidCurveGain { index, gain });
            }
            if let Some(previous) = curve.points().last()
                && distance <= previous.distance
            {
                return Err(Error::UnsortedCurvePoints {
                    index,
                    previous_distance: previous.distance,
                    distance,
                });
            }
            let Some(slot) = curve.points.get_mut(index) else {
                continue;
            };
            *slot = CurvePoint { distance, gain };
            curve.point_count = given;
        }
        if given > Self::MAX_POINTS {
            return Err(Error::TooManyCurvePoints {
                given,
                maximum: Self::MAX_POINTS,
            });
        }
        if given == 0 {
            return Err(Error::EmptyDistanceCurve);
        }
        Ok(curve)
    }

    /// The 25-point voice curve of the 1.0 plugin, built as
    /// `reference/tfar_1_0/ts/src/helpers.cpp:113-140` builds it for an emitter of `voice_volume`.
    ///
    /// The first point is `(0, 1)` and the last `(1.25, 0)`. In between, with
    /// `metres = voice_volume / 20`, the points at `k * 0.05` for `k = 1..=19` take
    /// `attenuation(metres * k)`, and the points at 1.05, 1.10, 1.15 and 1.20 take
    /// `attenuation(metres * 20)`, `(metres * 24)`, `(metres * 28)` and `(metres * 32)`.
    /// `attenuation` is the version's `volumeAttenuation` with the hearing state bound in; read
    /// the curve with `gain_at(distance, voice_volume)`.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidCurveGain`] when `attenuation` returns a value that is not finite and
    /// non-negative.
    pub fn tfar_voice(voice_volume: f64, attenuation: impl Fn(f64) -> f64) -> Result<Self> {
        let metres_per_step = voice_volume / 20.0;
        let gain_of = |metre_step: Option<f64>, index: usize| match metre_step {
            Some(steps) => attenuation(metres_per_step * steps),
            None if index == 0 => 1.0,
            None => 0.0,
        };
        Self::new(
            TFAR_VOICE_DISTANCES
                .iter()
                .zip(TFAR_VOICE_METRE_STEPS)
                .enumerate()
                .map(|(index, (distance, metre_step))| (*distance, gain_of(metre_step, index))),
        )
    }

    /// The points of the curve, in ascending distance.
    pub fn points(&self) -> &[CurvePoint] {
        self.points.get(..self.point_count).unwrap_or_default()
    }

    /// The gain at `distance` metres from an emitter whose curve is scaled by `scaler` metres.
    ///
    /// The normalised distance is `distance / scaler`. At or before the first point the first
    /// gain holds, at or past the last point the last gain holds, and in between the gain is
    /// `(1 - a) * g0 + a * g1` with `a` the position between the two neighbouring points, as
    /// FAudio's `ComputeCurveValue` computes it. A scaler that is not positive and finite, or a
    /// NaN distance, means the emitter reaches nowhere: the last gain is returned.
    pub fn gain_at(&self, distance: f64, scaler: f64) -> f64 {
        let points = self.points();
        let (Some(first), Some(last)) = (points.first(), points.last()) else {
            return 0.0;
        };
        let normalised = distance / scaler;
        if !(scaler.is_finite() && scaler > 0.0) || normalised.is_nan() {
            return last.gain;
        }
        if normalised <= first.distance {
            return first.gain;
        }
        for window in points.windows(2) {
            if let [before, after] = window
                && normalised < after.distance
            {
                let position = (normalised - before.distance) / (after.distance - before.distance);
                return (1.0 - position) * before.gain + position * after.gain;
            }
        }
        last.gain
    }
}
