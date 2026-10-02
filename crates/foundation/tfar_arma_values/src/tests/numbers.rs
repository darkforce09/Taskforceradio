//! Unit tests for `parse_number` and `parse_int`.

use super::*;

#[test]
fn parse_number_reads_the_numbers_arma_writes() {
    assert_eq!(parse_number("12").unwrap(), 12.0);
    assert_eq!(parse_number("-0.5").unwrap(), -0.5);
    assert_eq!(parse_number("1234.57").unwrap(), 1234.57);
    assert_eq!(parse_number("1.5e+006").unwrap(), 1_500_000.0);
    assert_eq!(parse_number("2.5e-005").unwrap(), 0.000_025);
    assert_eq!(parse_number("0").unwrap(), 0.0);
}

#[test]
fn parse_number_allows_leading_and_trailing_spaces() {
    assert_eq!(parse_number("  42.25 ").unwrap(), 42.25);
}

#[test]
fn a_malformed_number_is_an_error_not_a_crash() {
    for text in ["", " ", "abc", "1,5", "1.2.3", "12abc", "--1", "1 2", "[1]"] {
        let error = parse_number(text).unwrap_err();
        assert_eq!(
            error,
            Error::MalformedNumber {
                value: text.to_owned()
            },
            "text {text:?}"
        );
    }
}

#[test]
fn a_non_finite_number_is_an_error() {
    for text in ["inf", "-inf", "NaN", "infinity", "1e400"] {
        assert!(parse_number(text).is_err(), "text {text:?}");
    }
}

#[test]
fn parse_int_rounds_halves_away_from_zero_like_round_of_atof() {
    assert_eq!(parse_int("7").unwrap(), 7);
    assert_eq!(parse_int("6.5").unwrap(), 7);
    assert_eq!(parse_int("6.49").unwrap(), 6);
    assert_eq!(parse_int("-2.5").unwrap(), -3);
    assert_eq!(parse_int("-2.4").unwrap(), -2);
    assert_eq!(parse_int(" 1e+001 ").unwrap(), 10);
}

#[test]
fn parse_int_rejects_values_outside_the_i32_range() {
    assert_eq!(parse_int("2147483647").unwrap(), i32::MAX);
    assert_eq!(parse_int("-2147483648").unwrap(), i32::MIN);
    assert!(matches!(
        parse_int("2147483648"),
        Err(Error::IntegerOutOfRange { .. })
    ));
    assert!(matches!(
        parse_int("-1e10"),
        Err(Error::IntegerOutOfRange { .. })
    ));
}

#[test]
fn parse_int_reports_a_malformed_number() {
    assert!(matches!(
        parse_int("seven"),
        Err(Error::MalformedNumber { .. })
    ));
}
