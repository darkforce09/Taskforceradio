//! Unit tests for the foreground-window test.

use super::*;

#[test]
fn the_arma_class_name_matches_ignoring_case() {
    assert!(class_name_is_arma("Arma 3"));
    assert!(class_name_is_arma("ARMA"));
    assert!(class_name_is_arma("my arma window"));
}

#[test]
fn other_class_names_do_not_match() {
    assert!(!class_name_is_arma("TeamSpeak"));
    assert!(!class_name_is_arma("Arm a"));
    assert!(!class_name_is_arma(""));
}

#[test]
fn the_class_name_buffer_matches_the_original() {
    assert_eq!(CLASS_NAME_CAPACITY, 32);
}

#[cfg(not(windows))]
#[test]
fn off_windows_arma_is_never_in_front() {
    assert!(!foreground_window_is_arma());
}
