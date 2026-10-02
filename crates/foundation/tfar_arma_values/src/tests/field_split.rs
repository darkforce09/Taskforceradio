//! Unit tests for `split_fields`, against the `std::getline` loop's results.

use super::*;
use crate::separators::{FIELD, FREQUENCY_PART, SPEAKER_RECORD};

#[test]
fn split_fields_matches_the_getline_semantics_table() {
    let table: [(&str, &[&str]); 8] = [
        ("", &[]),
        ("a", &["a"]),
        ("a\t", &["a"]),
        ("a\t\tb", &["a", "", "b"]),
        ("\ta", &["", "a"]),
        ("a\t\t", &["a", ""]),
        ("\t", &[""]),
        ("POS\tnick\t[1,2,3]", &["POS", "nick", "[1,2,3]"]),
    ];
    for (input, expected) in table {
        assert_eq!(split_fields(input, FIELD), expected, "input {input:?}");
    }
}

#[test]
fn an_empty_input_gives_no_field() {
    assert_eq!(split_fields("", FIELD), Vec::<&str>::new());
}

#[test]
fn a_trailing_empty_field_is_dropped_but_inner_ones_are_kept() {
    assert_eq!(split_fields("87.6|7|", FREQUENCY_PART), ["87.6", "7"]);
    assert_eq!(split_fields("87.6||0", FREQUENCY_PART), ["87.6", "", "0"]);
}

#[test]
fn split_fields_uses_only_the_given_separator() {
    assert_eq!(split_fields("a\x0Bb\tc\x0B", SPEAKER_RECORD), ["a", "b\tc"]);
}

#[test]
fn split_keeping_every_field_keeps_the_trailing_empty_field() {
    assert_eq!(split_keeping_every_field("a|b|", '|'), ["a", "b", ""]);
    assert_eq!(split_keeping_every_field("", '|'), [""]);
}
