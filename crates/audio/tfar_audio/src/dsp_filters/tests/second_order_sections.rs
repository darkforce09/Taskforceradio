//! Unit tests for `SecondOrderSection` and `SecondOrderSections`.

use super::*;

const SAMPLE_RATE_HZ: f64 = 48_000.0;

fn pass_through() -> SecondOrderSection {
    SecondOrderSection::from_unnormalised([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]).unwrap()
}

#[test]
fn a_section_is_normalised_by_its_leading_denominator_coefficient() {
    let section = SecondOrderSection::from_unnormalised([2.0, 4.0, 6.0], [2.0, -1.0, 0.5]).unwrap();
    assert_eq!(section.coefficients(), [1.0, 2.0, 3.0, -0.5, 0.25]);
}

#[test]
fn a_zero_leading_denominator_coefficient_is_an_error() {
    let result = SecondOrderSection::from_unnormalised([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    assert!(matches!(result, Err(Error::DesignFailed { .. })));
}

#[test]
fn a_coefficient_that_is_not_finite_is_an_error() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let result = SecondOrderSection::from_unnormalised([1.0, bad, 0.0], [1.0, 0.0, 0.0]);
        assert!(matches!(result, Err(Error::DesignFailed { .. })), "{bad}");
    }
}

#[test]
fn an_empty_cascade_is_an_error() {
    let result = SecondOrderSections::new(Vec::new(), SAMPLE_RATE_HZ);
    assert!(matches!(result, Err(Error::DesignFailed { .. })));
}

#[test]
fn a_cascade_needs_a_valid_sample_rate() {
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let result = SecondOrderSections::new(vec![pass_through()], bad);
        assert!(
            matches!(result, Err(Error::InvalidSampleRate { .. })),
            "{bad}"
        );
    }
}

#[test]
fn a_pass_through_cascade_has_unit_gain_everywhere() {
    let cascade = SecondOrderSections::new(vec![pass_through(); 3], SAMPLE_RATE_HZ).unwrap();
    for frequency_hz in [0.0, 100.0, 1_000.0, 12_000.0, 24_000.0] {
        assert!((cascade.magnitude_at(frequency_hz) - 1.0).abs() < 1e-15);
    }
    assert_eq!(cascade.sections().len(), 3);
    assert_eq!(cascade.sample_rate_hz(), SAMPLE_RATE_HZ);
}

#[test]
fn the_cascade_gain_is_the_product_of_the_section_gains() {
    let half = SecondOrderSection::from_unnormalised([0.5, 0.0, 0.0], [1.0, 0.0, 0.0]).unwrap();
    let cascade = SecondOrderSections::new(vec![half, half], SAMPLE_RATE_HZ).unwrap();
    assert!((cascade.magnitude_at(1_000.0) - 0.25).abs() < 1e-15);
}

#[test]
fn a_one_sample_average_has_its_analytic_magnitude() {
    // H(z) = (1 + z^-1) / 2, |H| = |cos(ω / 2)|.
    let average = SecondOrderSection::from_unnormalised([0.5, 0.5, 0.0], [1.0, 0.0, 0.0]).unwrap();
    let cascade = SecondOrderSections::new(vec![average], SAMPLE_RATE_HZ).unwrap();
    for frequency_hz in [0.0, 3_000.0, 12_000.0, 20_000.0, 24_000.0] {
        let omega = core::f64::consts::TAU * frequency_hz / SAMPLE_RATE_HZ;
        let expected = (omega / 2.0).cos().abs();
        assert!((cascade.magnitude_at(frequency_hz) - expected).abs() < 1e-12);
    }
}
