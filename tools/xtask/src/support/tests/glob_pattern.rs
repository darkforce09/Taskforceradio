//! Tests for Contents globs.

use super::*;

#[test]
fn wildcards_classes_and_braces_match_whole_names() {
    assert!(matches("*.rs", "lib.rs"));
    assert!(!matches("*.rs", "lib.rs.bak"));
    assert!(matches("?.md", "a.md"));
    assert!(matches("[ab].txt", "b.txt"));
    assert!(!matches("[!ab].txt", "a.txt"));
    assert!(matches("{lib,main}.rs", "main.rs"));
    assert!(matches("v[0-9]_*", "v1_x"));
    assert!(!matches("lib", "lib.rs"));
}

#[test]
fn plain_names_are_not_globs() {
    assert!(!is_glob("lib.rs"));
    assert!(is_glob("*.rs"));
}
