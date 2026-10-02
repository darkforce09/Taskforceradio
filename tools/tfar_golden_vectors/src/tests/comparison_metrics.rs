//! Unit tests for `comparison_metrics`.

use super::*;

#[test]
fn the_rms_of_a_square_wave_is_its_amplitude() {
    assert!((root_mean_square(&[2.0, -2.0, 2.0, -2.0]) - 2.0).abs() < 1.0e-12);
    assert_eq!(root_mean_square(&[]), 0.0);
}

#[test]
fn the_relative_rms_error_scales_by_the_reference() {
    let error = relative_rms_error(&[2.0, -2.0], &[2.2, -2.2]).unwrap();
    assert!((error - 0.1).abs() < 1.0e-12);
}

#[test]
fn sequences_of_different_lengths_are_an_error() {
    assert_eq!(
        max_absolute_difference(&[1.0], &[1.0, 2.0]),
        Err(Error::LengthMismatch {
            reference: 1,
            candidate: 2
        })
    );
}

#[test]
fn the_largest_difference_and_the_decibel_distance_are_absolute() {
    assert_eq!(
        max_absolute_difference(&[0.0, 1.0], &[0.5, -1.0]).unwrap(),
        2.0
    );
    assert!((decibels(10.0) - 20.0).abs() < 1.0e-12);
    assert!((decibel_difference(1.0, 0.1) - 20.0).abs() < 1.0e-12);
}
