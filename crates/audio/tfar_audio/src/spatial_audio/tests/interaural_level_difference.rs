//! Unit tests for the simple interaural level difference in `tfar_audio::spatial_audio`.

use core::f64::consts::{FRAC_PI_2, PI};

use super::*;

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-6,
        "expected {expected}, got {actual}"
    );
}

/// The horizontal unit direction at the polar angle `radians` (counter-clockwise from east).
fn direction_at(radians: f64) -> Direction {
    Direction::from_array([radians.cos(), radians.sin(), 0.0]).unwrap()
}

#[test]
fn a_sound_ahead_gets_equal_gains() {
    for coefficient in [ILD_COEFFICIENT_0_9, ILD_COEFFICIENT_1_0] {
        let (left, right) = stereo_pan_gains(direction_at(FRAC_PI_2), Angle::ZERO, coefficient);
        assert_close(left, right);
        assert_close(left, 0.625);
        let (left, right) = stereo_pan_gains(direction_at(-FRAC_PI_2), Angle::ZERO, coefficient);
        assert_close(left, right);
    }
}

#[test]
fn hard_right_and_hard_left_have_the_original_values() {
    // dir = 0: cos = 1, so left = 0.625 - c and right = 0.625 + c.
    let (left, right) = stereo_pan_gains(direction_at(0.0), Angle::ZERO, ILD_COEFFICIENT_0_9);
    assert_close(left, 0.25);
    assert_close(right, 1.0);
    // dir = π: cos = -1, the mirror image.
    let (left, right) = stereo_pan_gains(direction_at(PI), Angle::ZERO, ILD_COEFFICIENT_0_9);
    assert_close(left, 1.0);
    assert_close(right, 0.25);
    let (left, right) = stereo_pan_gains(direction_at(0.0), Angle::ZERO, ILD_COEFFICIENT_1_0);
    assert_close(left, 0.625 - ILD_COEFFICIENT_1_0);
    assert_close(right, 0.625 + ILD_COEFFICIENT_1_0);
}

#[test]
fn mirrored_directions_swap_the_gains() {
    for polar in [0.3, 1.0, 2.0, -0.7] {
        let (left, right) = stereo_pan_gains(direction_at(polar), Angle::ZERO, ILD_COEFFICIENT_1_0);
        let (mirror_left, mirror_right) =
            stereo_pan_gains(direction_at(PI - polar), Angle::ZERO, ILD_COEFFICIENT_1_0);
        assert_close(left, mirror_right);
        assert_close(right, mirror_left);
    }
}

#[test]
fn the_view_angle_is_added_to_the_polar_angle() {
    // Polar angle π/2 plus a view angle of -π/2 is dir = 0: hard right.
    let (left, right) = stereo_pan_gains(
        direction_at(FRAC_PI_2),
        Angle::from_radians(-FRAC_PI_2),
        ILD_COEFFICIENT_0_9,
    );
    assert_close(left, 0.25);
    assert_close(right, 1.0);
}

#[test]
fn a_negative_sum_is_wrapped_like_its_positive_equivalent() {
    // Polar -170° plus view -100° is -270°, which wraps to 90°: straight ahead, equal gains.
    let polar = (-170.0_f64).to_radians();
    let view = Angle::from_degrees(-100.0);
    let (left, right) = stereo_pan_gains(direction_at(polar), view, ILD_COEFFICIENT_1_0);
    assert_close(left, right);
    assert_close(left, 0.625);
    // Polar -150° plus view -200° is -350°, which wraps to 10°: nearly hard right.
    let (left, right) = stereo_pan_gains(
        direction_at((-150.0_f64).to_radians()),
        Angle::from_degrees(-200.0),
        ILD_COEFFICIENT_1_0,
    );
    let (expected_left, expected_right) = stereo_pan_gains(
        direction_at(10.0_f64.to_radians()),
        Angle::ZERO,
        ILD_COEFFICIENT_1_0,
    );
    assert_close(left, expected_left);
    assert_close(right, expected_right);
    assert!(right > left);
}

#[test]
fn apply_pan_scales_each_channel_by_its_gain() {
    let mut stereo = StereoBuffer::with_frames(2);
    stereo
        .as_mut_slice()
        .copy_from_slice(&[1000, 1000, -2000, 400]);
    apply_pan(&mut stereo, (0.25, 1.0));
    assert_eq!(stereo.as_slice(), &[250, 1000, -500, 400]);
}
