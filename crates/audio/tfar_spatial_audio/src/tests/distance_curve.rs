//! Unit tests for `DistanceCurve` in `tfar_spatial_audio`.

use super::*;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-12,
        "expected {expected}, got {actual}"
    );
}

fn three_point_curve() -> DistanceCurve {
    DistanceCurve::new([(0.0, 1.0), (0.5, 0.5), (1.0, 0.0)]).unwrap()
}

#[test]
fn the_gain_between_two_points_is_interpolated_linearly() {
    let curve = three_point_curve();
    assert_close(curve.gain_at(0.25, 1.0), 0.75);
    assert_close(curve.gain_at(0.75, 1.0), 0.25);
    assert_close(curve.gain_at(0.1, 1.0), 0.9);
}

#[test]
fn the_distance_is_normalised_by_the_scaler() {
    let curve = three_point_curve();
    assert_close(curve.gain_at(10.0, 40.0), 0.75);
    assert_close(curve.gain_at(30.0, 40.0), 0.25);
}

#[test]
fn a_gain_on_a_point_is_that_points_gain() {
    let curve = three_point_curve();
    assert_close(curve.gain_at(0.0, 1.0), 1.0);
    assert_close(curve.gain_at(0.5, 1.0), 0.5);
    assert_close(curve.gain_at(1.0, 1.0), 0.0);
}

#[test]
fn the_last_gain_holds_past_the_end() {
    let curve = DistanceCurve::new([(0.0, 1.0), (1.0, 0.2)]).unwrap();
    assert_close(curve.gain_at(1.5, 1.0), 0.2);
    assert_close(curve.gain_at(1e9, 1.0), 0.2);
    assert_close(curve.gain_at(f64::INFINITY, 1.0), 0.2);
}

#[test]
fn the_first_gain_holds_before_the_start() {
    let curve = DistanceCurve::new([(0.2, 0.8), (1.0, 0.0)]).unwrap();
    assert_close(curve.gain_at(0.1, 1.0), 0.8);
    assert_close(curve.gain_at(-3.0, 1.0), 0.8);
}

#[test]
fn a_single_point_curve_is_constant() {
    let curve = DistanceCurve::new([(0.0, 0.4)]).unwrap();
    for distance in [0.0, 0.3, 10.0] {
        assert_close(curve.gain_at(distance, 1.0), 0.4);
    }
}

#[test]
fn a_scaler_that_is_not_positive_and_finite_gives_the_last_gain() {
    let curve = three_point_curve();
    for scaler in [0.0, -5.0, f64::NAN, f64::INFINITY] {
        assert_close(curve.gain_at(0.0, scaler), 0.0);
    }
    assert_close(curve.gain_at(f64::NAN, 1.0), 0.0);
}

#[test]
fn unsorted_points_are_an_error() {
    assert_eq!(
        DistanceCurve::new([(0.0, 1.0), (0.6, 0.5), (0.4, 0.0)]),
        Err(Error::UnsortedCurvePoints {
            index: 2,
            previous_distance: 0.6,
            distance: 0.4,
        })
    );
}

#[test]
fn a_repeated_distance_is_an_error() {
    assert!(matches!(
        DistanceCurve::new([(0.0, 1.0), (0.5, 0.5), (0.5, 0.2)]),
        Err(Error::UnsortedCurvePoints { index: 2, .. })
    ));
}

#[test]
fn an_empty_curve_is_an_error() {
    assert_eq!(DistanceCurve::new([]), Err(Error::EmptyDistanceCurve));
}

#[test]
fn too_many_points_are_an_error() {
    let points =
        (0..=DistanceCurve::MAX_POINTS).map(|step| (f64::from(u32::try_from(step).unwrap()), 1.0));
    assert_eq!(
        DistanceCurve::new(points),
        Err(Error::TooManyCurvePoints {
            given: DistanceCurve::MAX_POINTS + 1,
            maximum: DistanceCurve::MAX_POINTS,
        })
    );
}

#[test]
fn non_finite_or_negative_values_are_an_error() {
    assert!(matches!(
        DistanceCurve::new([(f64::NAN, 1.0)]),
        Err(Error::InvalidCurveDistance { index: 0, .. })
    ));
    assert_eq!(
        DistanceCurve::new([(0.0, 1.0), (-1.0, 1.0)]),
        Err(Error::InvalidCurveDistance {
            index: 1,
            distance: -1.0,
        })
    );
    assert_eq!(
        DistanceCurve::new([(0.0, f64::INFINITY)]),
        Err(Error::InvalidCurveGain {
            index: 0,
            gain: f64::INFINITY,
        })
    );
    assert_eq!(
        DistanceCurve::new([(0.0, -0.1)]),
        Err(Error::InvalidCurveGain {
            index: 0,
            gain: -0.1,
        })
    );
}

/// The attenuation of `reference/tfar_1_0/ts/src/helpers.hpp:122` for a player who can hear,
/// with `max_audible` metres of range: 1 within a metre, then 10^(-1.5 * d / max), cut below
/// 0.001.
fn voice_attenuation(max_audible: f64) -> impl Fn(f64) -> f64 {
    move |distance| {
        if distance <= 1.0 {
            return 1.0;
        }
        let gain = 10.0_f64.powf((distance / (max_audible * 2.0)) * -60.0 / 20.0);
        if gain < 0.001 { 0.0 } else { gain.min(1.0) }
    }
}

#[test]
fn the_tfar_voice_curve_has_the_original_25_points() {
    let voice_volume = 40.0;
    let attenuation = voice_attenuation(voice_volume);
    let curve = DistanceCurve::tfar_voice(voice_volume, &attenuation).unwrap();
    let points = curve.points();
    assert_eq!(points.len(), 25);
    let distances: Vec<f64> = points.iter().map(|point| point.distance).collect();
    assert_eq!(
        distances,
        [
            0.0, 0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40, 0.45, 0.50, 0.55, 0.60, 0.65,
            0.70, 0.75, 0.80, 0.85, 0.90, 0.95, 1.05, 1.10, 1.15, 1.20, 1.25,
        ]
    );
    assert_close(points[0].gain, 1.0);
    // volumePerMeter = 40 / 20 = 2 m: point k (k = 1..=19) is the attenuation at 2k metres.
    for (k, point) in points.iter().enumerate().take(20).skip(1) {
        assert_close(
            point.gain,
            attenuation(2.0 * f64::from(u8::try_from(k).unwrap())),
        );
    }
    for (point, metres) in points[20..24].iter().zip([40.0, 48.0, 56.0, 64.0]) {
        assert_close(point.gain, attenuation(metres));
    }
    assert_close(points[24].gain, 0.0);
}

#[test]
fn the_tfar_voice_curve_is_monotone_non_increasing_for_a_monotone_attenuation() {
    for voice_volume in [5.0, 20.0, 40.0, 60.0] {
        let curve =
            DistanceCurve::tfar_voice(voice_volume, voice_attenuation(voice_volume)).unwrap();
        for pair in curve.points().windows(2) {
            assert!(
                pair[1].gain <= pair[0].gain,
                "voice volume {voice_volume}: {pair:?} rises"
            );
        }
        let mut previous = f64::INFINITY;
        for step in 0..=400 {
            let distance = f64::from(step) * voice_volume / 200.0;
            let gain = curve.gain_at(distance, voice_volume);
            assert!(gain <= previous, "the gain rises at {distance} m");
            assert!((0.0..=1.0).contains(&gain));
            previous = gain;
        }
        assert_close(curve.gain_at(voice_volume * 1.25, voice_volume), 0.0);
        assert_close(curve.gain_at(0.0, voice_volume), 1.0);
    }
}

#[test]
fn an_attenuation_returning_nan_is_an_error() {
    assert!(matches!(
        DistanceCurve::tfar_voice(40.0, |_| f64::NAN),
        Err(Error::InvalidCurveGain { index: 1, .. })
    ));
}
