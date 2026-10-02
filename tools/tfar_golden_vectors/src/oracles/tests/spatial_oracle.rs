//! Unit tests for `spatial_oracle`: scene checks and F3DAudio's plain properties.

use super::*;

fn scene(listener_x: f32, volume_curve: Vec<(f32, f32)>) -> OracleSpatialScene {
    OracleSpatialScene {
        listener_position: [listener_x, 0.0, 0.0],
        listener_front: [0.0, 0.0, 1.0],
        listener_top: [0.0, 1.0, 0.0],
        emitter_position: [0.0, 0.0, 0.0],
        emitter_front: [1.0, 0.0, 0.0],
        emitter_top: [0.0, 1.0, 0.0],
        volume_curve,
        curve_distance_scaler: 10.0,
        cone: None,
    }
}

#[test]
fn a_scaler_of_zero_and_a_curve_beyond_the_maximum_are_refused() {
    let mut zero_scaler = scene(1.0, Vec::new());
    zero_scaler.curve_distance_scaler = 0.0;
    assert!(f3daudio_calculate(&zero_scaler).is_err());
    let long_curve = (0..=ORACLE_MAX_CURVE_POINTS)
        .map(|index| (f32::from(u8::try_from(index).unwrap()) / 64.0, 1.0))
        .collect();
    assert!(matches!(
        f3daudio_calculate(&scene(1.0, long_curve)),
        Err(Error::TooManyCurvePoints { given: 33, .. })
    ));
}

#[test]
fn the_default_inverse_law_halves_the_volume_at_twice_the_scaler() {
    let result = f3daudio_calculate(&scene(20.0, Vec::new())).unwrap();
    assert!((result.volume - 0.5).abs() < 1.0e-6);
    assert_eq!(result.failed_parameter_checks, 0);
    assert!((result.emitter_to_listener_distance - 20.0).abs() < 1.0e-6);
}

#[test]
fn a_valid_curve_is_interpolated_and_breaks_no_parameter_check() {
    let result = f3daudio_calculate(&scene(5.0, vec![(0.0, 1.0), (1.0, 0.0)])).unwrap();
    assert!((result.volume - 0.5).abs() < 1.0e-6);
    assert_eq!(result.failed_parameter_checks, 0);
}

#[test]
fn a_curve_reaching_past_distance_one_breaks_the_x3daudio_range_checks() {
    let result = f3daudio_calculate(&scene(5.0, vec![(0.0, 1.0), (1.25, 0.0)])).unwrap();
    assert_eq!(result.failed_parameter_checks, 2);
}
