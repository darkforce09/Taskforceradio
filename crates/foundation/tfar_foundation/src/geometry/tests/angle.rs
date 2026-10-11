//! Unit tests for `Angle`.

use core::f64::consts::{FRAC_PI_2, PI, TAU};

use super::*;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-12,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn degrees_convert_to_radians_and_back() {
    assert_close(Angle::from_degrees(180.0).radians(), PI);
    assert_close(Angle::from_degrees(-90.0).radians(), -FRAC_PI_2);
    assert_close(Angle::from_radians(FRAC_PI_2).to_degrees(), 90.0);
    assert_close(Angle::from_degrees(21.5).to_degrees(), 21.5);
}

#[test]
fn angles_add_subtract_and_negate_in_radians() {
    let quarter = Angle::from_radians(FRAC_PI_2);
    assert_close((quarter + quarter).radians(), PI);
    assert_close((quarter - quarter).radians(), 0.0);
    assert_close((-quarter).radians(), -FRAC_PI_2);
}

#[test]
fn cosine_and_sine_use_radians() {
    assert_close(Angle::from_degrees(60.0).cos(), 0.5);
    assert_close(Angle::from_degrees(30.0).sin(), 0.5);
}

#[test]
fn normalized_signed_keeps_angles_already_in_range() {
    for radians in [0.0, 1.0, -1.0, 3.0, -3.0, PI] {
        assert_close(
            Angle::from_radians(radians).normalized_signed().radians(),
            radians,
        );
    }
}

#[test]
fn normalized_signed_maps_minus_pi_to_pi() {
    assert_close(Angle::from_radians(-PI).normalized_signed().radians(), PI);
}

#[test]
fn normalized_signed_wraps_angles_below_minus_pi() {
    assert_close(
        Angle::from_radians(-PI - 0.5).normalized_signed().radians(),
        PI - 0.5,
    );
    assert_close(Angle::from_radians(-TAU).normalized_signed().radians(), 0.0);
    assert_close(
        Angle::from_degrees(-270.0).normalized_signed().to_degrees(),
        90.0,
    );
}

#[test]
fn normalized_signed_wraps_angles_between_pi_and_two_pi() {
    assert_close(
        Angle::from_degrees(190.0).normalized_signed().to_degrees(),
        -170.0,
    );
    assert_close(
        Angle::from_degrees(359.0).normalized_signed().to_degrees(),
        -1.0,
    );
}

#[test]
fn normalized_signed_wraps_many_turns() {
    assert_close(
        Angle::from_degrees(3.0 * 360.0 + 45.0)
            .normalized_signed()
            .to_degrees(),
        45.0,
    );
    assert_close(
        Angle::from_degrees(-5.0 * 360.0 - 45.0)
            .normalized_signed()
            .to_degrees(),
        -45.0,
    );
}

#[test]
fn normalized_signed_never_leaves_the_half_open_range() {
    let mut degrees = -1000.0;
    while degrees <= 1000.0 {
        let wrapped = Angle::from_degrees(degrees).normalized_signed().radians();
        assert!(
            wrapped > -PI && wrapped <= PI,
            "{degrees} wrapped to {wrapped}"
        );
        degrees += 0.25;
    }
    let tiny_negative = Angle::from_radians(-1e-300).normalized_signed().radians();
    assert!(tiny_negative > -PI && tiny_negative <= PI);
}
