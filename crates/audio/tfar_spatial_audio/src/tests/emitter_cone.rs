//! Unit tests for `EmitterCone` in `tfar_spatial_audio`.

use super::*;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-12,
        "expected {expected}, got {actual}"
    );
}

/// The horizontal unit direction at `degrees` counter-clockwise from east.
fn direction_at(degrees: f64) -> Direction {
    let radians = degrees.to_radians();
    Direction::from_array([radians.cos(), radians.sin(), 0.0]).unwrap()
}

#[test]
fn the_tfar_voice_cone_has_the_original_values() {
    let cone = EmitterCone::TFAR_VOICE;
    assert_close(cone.inner_angle().to_degrees(), 90.0);
    assert_close(cone.outer_angle().to_degrees(), 135.0);
    assert_close(cone.inner_volume(), 1.8);
    assert_close(cone.outer_volume(), 1.0);
    assert_eq!(
        EmitterCone::new(
            Angle::from_degrees(90.0),
            Angle::from_degrees(135.0),
            1.8,
            1.0
        )
        .map(EmitterCone::inner_volume),
        Ok(1.8)
    );
}

#[test]
fn inside_half_the_inner_angle_the_inner_volume_applies() {
    let cone = EmitterCone::TFAR_VOICE;
    let front = direction_at(0.0);
    for degrees in [0.0, 20.0, -44.0, 45.0] {
        assert_close(cone.gain(front, direction_at(degrees)), 1.8);
    }
}

#[test]
fn beyond_half_the_outer_angle_the_outer_volume_applies() {
    let cone = EmitterCone::TFAR_VOICE;
    let front = direction_at(0.0);
    for degrees in [67.6, 90.0, 180.0, -100.0] {
        assert_close(cone.gain(front, direction_at(degrees)), 1.0);
    }
}

#[test]
fn between_the_half_angles_the_volume_is_interpolated() {
    let cone = EmitterCone::TFAR_VOICE;
    let front = direction_at(0.0);
    // Half inner 45°, half outer 67.5°: 56.25° is halfway.
    assert_close(cone.gain(front, direction_at(56.25)), 1.4);
    assert_close(cone.gain(front, direction_at(-56.25)), 1.4);
    // A quarter of the way.
    assert_close(cone.gain(front, direction_at(50.625)), 1.6);
}

#[test]
fn the_cone_follows_the_emitter_front_in_three_dimensions() {
    let cone = EmitterCone::TFAR_VOICE;
    let up = Direction::from_array([0.0, 0.0, 1.0]).unwrap();
    assert_close(cone.gain(up, up), 1.8);
    assert_close(cone.gain(up, direction_at(0.0)), 1.0);
}

#[test]
fn a_listener_at_the_emitter_gets_the_inner_volume() {
    let cone = EmitterCone::TFAR_VOICE;
    assert_close(cone.gain(direction_at(0.0), Direction::ZERO), 1.8);
}

#[test]
fn a_zero_front_counts_as_ninety_degrees() {
    let cone = EmitterCone::TFAR_VOICE;
    assert_close(cone.gain(Direction::ZERO, direction_at(0.0)), 1.0);
    let wide = EmitterCone::new(
        Angle::from_degrees(180.0),
        Angle::from_degrees(360.0),
        2.0,
        0.0,
    )
    .unwrap();
    assert_close(wide.gain(Direction::ZERO, direction_at(0.0)), 2.0);
}

#[test]
fn equal_inner_and_outer_angles_switch_without_dividing_by_zero() {
    let cone = EmitterCone::new(
        Angle::from_degrees(90.0),
        Angle::from_degrees(90.0),
        1.5,
        0.5,
    )
    .unwrap();
    let front = direction_at(0.0);
    assert_close(cone.gain(front, direction_at(44.0)), 1.5);
    assert_close(cone.gain(front, direction_at(46.0)), 0.5);
}

#[test]
fn invalid_angles_are_an_error() {
    let cases = [(-1.0, 90.0), (100.0, 90.0), (90.0, 361.0), (f64::NAN, 90.0)];
    for (inner, outer) in cases {
        assert!(
            matches!(
                EmitterCone::new(
                    Angle::from_degrees(inner),
                    Angle::from_degrees(outer),
                    1.0,
                    1.0
                ),
                Err(Error::InvalidConeAngles { .. })
            ),
            "inner {inner}, outer {outer}"
        );
    }
}

#[test]
fn invalid_volumes_are_an_error() {
    let inner = Angle::from_degrees(90.0);
    let outer = Angle::from_degrees(135.0);
    assert_eq!(
        EmitterCone::new(inner, outer, 2.5, 1.0),
        Err(Error::InvalidConeVolume {
            which: "inner",
            volume: 2.5,
        })
    );
    assert_eq!(
        EmitterCone::new(inner, outer, 1.0, -0.5),
        Err(Error::InvalidConeVolume {
            which: "outer",
            volume: -0.5,
        })
    );
    assert!(EmitterCone::new(inner, outer, f64::NAN, 1.0).is_err());
}
