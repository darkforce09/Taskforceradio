//! Unit tests for `parameter_checks`.

use super::*;

#[test]
fn a_positive_finite_value_passes_and_zero_nan_and_infinity_fail() {
    assert!(check_positive("width", 1.0e-9).is_ok());
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(check_positive("width", value).is_err(), "{value}");
    }
}

#[test]
fn a_frequency_must_lie_strictly_between_zero_and_nyquist() {
    assert!(check_below_nyquist("cutoff", 23_999.0, 24_000.0).is_ok());
    for value in [0.0, 24_000.0, 30_000.0, f64::NAN] {
        assert!(
            check_below_nyquist("cutoff", value, 24_000.0).is_err(),
            "{value}"
        );
    }
}
