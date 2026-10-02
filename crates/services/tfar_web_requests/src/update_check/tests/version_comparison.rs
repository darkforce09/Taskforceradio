//! Unit tests for the version shape check and the newer-than comparison.

use super::*;

#[test]
fn a_newer_published_version_is_an_update() {
    assert!(is_update_available("1.0.0.1", "1.0.0.2"));
    assert!(is_update_available("1.0.0.1", "1.1"));
    assert!(is_update_available("0.9.12", "0.9.13"));
    assert!(is_update_available("0.9.12", "1.0.0.0"));
}

#[test]
fn the_same_version_is_no_update() {
    assert!(!is_update_available("1.0.0.1", "1.0.0.1"));
    assert!(!is_update_available("0.9.12", "0.9.12"));
}

#[test]
fn an_older_published_version_is_no_update() {
    assert!(!is_update_available("1.0.0.1", "1.0.0.0"));
    assert!(!is_update_available("1.0.0.1", "0.9.12"));
}

#[test]
fn components_compare_as_numbers_so_0_9_12_is_newer_than_0_9_3() {
    // A digit sum that multiplies by ten at each dot reads both as 93 (0, 0, 9, 90, 91, 93 and
    // 0, 0, 9, 90, 93); a component-wise comparison sees 12 > 3.
    assert!(is_update_available("0.9.3", "0.9.12"));
    assert!(!is_update_available("0.9.12", "0.9.3"));
    assert!(is_update_available("0.9.12", "0.10.0"));
}

#[test]
fn every_component_counts_so_a_longer_newer_version_is_an_update() {
    assert!(is_update_available("1.0", "1.0.0.1"));
    assert!(!is_update_available("1.0.0.1", "1.0"));
}

#[test]
fn a_body_with_surrounding_whitespace_still_reads_as_its_version() {
    assert!(is_update_available("0.9.12", " 0.9.13\r\n"));
    assert!(!is_update_available("0.9.12", "0.9.12\n"));
}

#[test]
fn garbage_and_empty_bodies_are_no_update() {
    for body in [
        "",
        "   \n",
        "garbage",
        "<html>404: Not Found</html>",
        "1.0.x",
        "1..0",
        ".1",
        "1.",
        "-1.0",
        "v1.0.0.2",
        "1.0.0.2 beta",
    ] {
        assert!(
            !is_update_available("1.0.0.1", body),
            "{body:?} counted as newer"
        );
    }
}

#[test]
fn a_current_version_that_is_not_dotted_never_sees_an_update() {
    assert!(!is_update_available("dev", "9.9.9"));
}

#[test]
fn the_shape_check_accepts_only_dotted_digit_groups() {
    assert!(is_version_string("0.9.12"));
    assert!(is_version_string("1.0.0.1"));
    assert!(is_version_string("7"));
    assert!(!is_version_string(""));
    assert!(!is_version_string("1.0 "));
    assert!(!is_version_string("1,0"));
    assert!(!is_version_string("\u{661}.\u{660}"));
}
