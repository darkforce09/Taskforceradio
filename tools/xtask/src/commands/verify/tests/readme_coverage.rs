//! Fixture tests for the `readme-coverage` gate.

use super::*;
use crate::fixture_tree::{FixtureTree, described};

const FOUNDATION_README: &str = "crates/foundation/README.md";

#[test]
fn the_sample_workspace_holds() {
    let fixture = FixtureTree::sample_workspace();
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn a_folder_without_a_readme_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.remove(FOUNDATION_README);
    let found = described(&check(&fixture.repository()).unwrap());
    assert_eq!(
        found,
        vec!["crates/foundation/:1: this folder has no README.md".to_owned()]
    );
}

#[test]
fn exempt_folders_need_no_readme() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write(
        "crates/foundation/tfar_geometry/src/tests/fixtures/data.txt",
        "data\n",
    );
    fixture.write(
        "crates/foundation/tfar_geometry/.config/settings.toml",
        "a = 1\n",
    );
    fixture.replace(
        "crates/foundation/tfar_geometry/README.md",
        "├── Cargo.toml",
        "├── .config/    tool settings\n├── Cargo.toml",
    );
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn an_unlisted_child_and_a_stale_entry_both_fail() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write("crates/foundation/notes.txt", "notes\n");
    fixture.replace(
        FOUNDATION_README,
        "└── tfar_geometry/",
        "├── gone.rs  a file that is not there\n└── tfar_geometry/",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("the file `notes.txt` is not listed")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|line| line.contains("the entry `gone.rs` names no file")),
        "{found:?}"
    );
}

#[test]
fn a_folder_listed_as_a_file_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        FOUNDATION_README,
        "└── tfar_geometry/  ",
        "└── tfar_geometry   ",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("the folder `tfar_geometry/` is not listed")),
        "{found:?}"
    );
}

#[test]
fn a_nested_entry_and_a_wrong_root_line_fail() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(FOUNDATION_README, "crates/foundation/\n", "foundation/\n");
    fixture.replace(
        FOUNDATION_README,
        "└── tfar_geometry/  a sample child folder\n",
        "└── tfar_geometry/  a sample child folder\n│   └── Cargo.toml  nested\n",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("expected `crates/foundation/`")),
        "{found:?}"
    );
    assert!(
        found.iter().any(|line| line.contains("nested or indented")),
        "{found:?}"
    );
}

#[test]
fn a_glob_entry_matches_every_child_it_names() {
    let fixture = FixtureTree::sample_workspace();
    let src_readme = "crates/foundation/tfar_geometry/src/README.md";
    fixture.replace(src_readme, "├── crate_role.rs  `CRATE_ROLE`: what the crate is for, in one sentence\n├── error.rs       `Error`, the `thiserror` enum, and `Result`\n", "├── {crate_role,error}.rs  the role and the error\n");
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}
