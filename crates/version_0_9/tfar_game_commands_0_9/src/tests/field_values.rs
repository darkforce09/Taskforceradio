//! Unit tests for the shared field readers.

use super::*;

#[test]
fn a_rounded_integer_rounds_halves_away_from_zero_like_parse_arma_number_to_int() {
    // `parseArmaNumberToInt` is `round(atof)`, reference/tfar_0_9_12/ts/src/plugin.cpp:112-120.
    assert_eq!(rounded_integer("FREQ", "f", "2.5").unwrap(), 3);
    assert_eq!(rounded_integer("FREQ", "f", "-2.5").unwrap(), -3);
    assert_eq!(rounded_integer("FREQ", "f", "1.5e+006").unwrap(), 1_500_000);
}

#[test]
fn a_truncated_integer_truncates_toward_zero_like_stoi() {
    // `std::stoi` (plugin.cpp:1810) and `(int)std::atof` (plugin.cpp:1883) truncate.
    assert_eq!(truncated_integer("POS", "f", "3.7").unwrap(), 3);
    assert_eq!(truncated_integer("POS", "f", "-3.7").unwrap(), -3);
    assert_eq!(truncated_integer("POS", "f", "12").unwrap(), 12);
}

#[test]
fn a_truncated_integer_outside_the_i32_range_is_malformed() {
    let error = truncated_integer("POS", "terrain", "1e20").unwrap_err();
    assert!(matches!(
        error,
        Error::Malformed {
            command: "POS",
            field: "terrain",
            source: tfar_arma_values::Error::IntegerOutOfRange { .. }
        }
    ));
}

#[test]
fn a_malformed_number_names_its_command_and_field() {
    let error = number("FREQ", "waves_level", "abc").unwrap_err();
    assert!(matches!(
        error,
        Error::Malformed {
            command: "FREQ",
            field: "waves_level",
            source: tfar_arma_values::Error::MalformedNumber { .. }
        }
    ));
}

#[test]
fn only_the_exact_text_true_is_true_like_is_true() {
    // `isTrue` compares with "true", plugin.cpp:1473-1476.
    assert!(is_true("true"));
    assert!(!is_true("True"));
    assert!(!is_true("false"));
    assert!(!is_true(" true"));
    assert!(!is_true(""));
}

#[test]
fn exact_fields_returns_the_fields_or_names_the_counts() {
    let [a, b] = exact_fields::<2>("TRACK", &["TRACK", "x"]).unwrap();
    assert_eq!((a, b), ("TRACK", "x"));
    assert_eq!(
        exact_fields::<2>("TRACK", &["TRACK"]).unwrap_err(),
        Error::WrongFieldCount {
            command: "TRACK",
            expected: 2,
            found: 1
        }
    );
}
