//! Unit tests for the design parameter checks.

use super::*;

const SAMPLE_RATE_HZ: f64 = 48_000.0;

#[test]
fn a_finite_positive_sample_rate_is_accepted() {
    assert!(check_sample_rate(48_000.0).is_ok());
    assert!(check_sample_rate(1.0).is_ok());
}

#[test]
fn a_zero_negative_or_non_finite_sample_rate_is_refused() {
    for bad in [0.0, -48_000.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(
            matches!(check_sample_rate(bad), Err(Error::InvalidSampleRate { .. })),
            "{bad}"
        );
    }
}

#[test]
fn a_frequency_strictly_inside_the_band_is_accepted_and_normalised() {
    let normalised = check_design_frequency("cutoff", 12_000.0, SAMPLE_RATE_HZ).unwrap();
    assert_eq!(normalised, 0.5);
    assert!(check_design_frequency("cutoff", 0.001, SAMPLE_RATE_HZ).is_ok());
    assert!(check_design_frequency("cutoff", 23_999.999, SAMPLE_RATE_HZ).is_ok());
}

#[test]
fn a_frequency_at_or_beyond_zero_or_nyquist_is_refused_with_its_role() {
    for bad in [0.0, -1.0, 24_000.0, 30_000.0, f64::NAN, f64::INFINITY] {
        match check_design_frequency("upper band edge", bad, SAMPLE_RATE_HZ) {
            Err(Error::FrequencyOutOfRange {
                frequency_role,
                nyquist_hz,
                ..
            }) => {
                assert_eq!(frequency_role, "upper band edge");
                assert_eq!(nyquist_hz, 24_000.0);
            }
            other => panic!("{bad}: {other:?}"),
        }
    }
}

#[test]
fn a_positive_finite_quality_is_accepted_and_anything_else_refused() {
    assert!(check_quality(0.707).is_ok());
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(
            matches!(check_quality(bad), Err(Error::InvalidQuality { .. })),
            "{bad}"
        );
    }
}

#[test]
fn an_order_outside_one_to_the_maximum_is_refused() {
    assert!(check_order(1).is_ok());
    assert!(check_order(MAX_BUTTERWORTH_ORDER).is_ok());
    for bad in [0, MAX_BUTTERWORTH_ORDER + 1, usize::MAX] {
        assert!(
            matches!(check_order(bad), Err(Error::InvalidOrder { order, .. }) if order == bad),
            "{bad}"
        );
    }
}
