//! Unit tests for `sanitize_nickname`.

use super::*;

#[test]
fn a_name_without_edge_spaces_is_unchanged() {
    assert_eq!(sanitize_nickname("Sam"), "Sam");
    assert_eq!(sanitize_nickname("Sam Dool"), "Sam Dool");
    assert_eq!(sanitize_nickname(""), "");
}

#[test]
fn leading_spaces_become_underscores() {
    assert_eq!(sanitize_nickname("  Sam"), "__Sam");
}

#[test]
fn trailing_spaces_become_underscores() {
    assert_eq!(sanitize_nickname("Sam   "), "Sam___");
}

#[test]
fn both_edges_are_replaced_and_inner_spaces_stay() {
    assert_eq!(sanitize_nickname(" Sam  Dool "), "_Sam  Dool_");
}

#[test]
fn a_name_of_spaces_only_becomes_underscores() {
    assert_eq!(sanitize_nickname("   "), "___");
}

#[test]
fn multibyte_names_keep_their_characters() {
    assert_eq!(sanitize_nickname(" Jörg "), "_Jörg_");
}
