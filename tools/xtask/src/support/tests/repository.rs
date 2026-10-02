//! Tests for the repository listing.

use super::*;
use crate::fixture_tree::FixtureTree;

#[test]
fn the_listing_holds_tracked_and_untracked_files_but_not_ignored_ones() {
    let fixture = FixtureTree::empty();
    fixture.write(".gitignore", "/target/\n");
    fixture.write("crates/a/src/lib.rs", "\n");
    fixture.write("target/debug/out", "\n");
    let repository = fixture.repository();
    assert_eq!(
        repository.files().unwrap(),
        [".gitignore", "crates/a/src/lib.rs"]
    );
    assert!(repository.folders().unwrap().contains("crates/a/src"));
    assert!(repository.path_exists("crates/a/").unwrap());
    assert!(repository.path_exists("target/debug/out").unwrap());
    assert!(!repository.path_exists("crates/b").unwrap());
}

#[test]
fn path_helpers_split_names_and_folders() {
    assert_eq!(file_name("a/b/c.rs"), "c.rs");
    assert_eq!(parent_folder("a/b/c.rs"), "a/b");
    assert_eq!(parent_folder("c.rs"), "");
    assert!(has_folder_named("a/tests/c.rs", "tests"));
    assert!(!has_folder_named("tests.rs", "tests"));
}
