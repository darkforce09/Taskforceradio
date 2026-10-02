//! Fixture tests for the `editorconfig` gate.

use super::*;
use crate::fixture_tree::{FixtureTree, described};

#[test]
fn the_sample_workspace_holds() {
    let fixture = FixtureTree::sample_workspace();
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn each_broken_rule_is_reported() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write(
        "crates/foundation/tfar_geometry/src/broken.rs",
        "fn a() {} \n\tfn b() {}\r\nfn c() {}",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    let path = "crates/foundation/tfar_geometry/src/broken.rs";
    assert!(
        found.contains(&format!("{path}:1: trailing whitespace")),
        "{found:?}"
    );
    assert!(
        found.contains(&format!(
            "{path}:2: tab indentation; this file type indents with spaces"
        )),
        "{found:?}"
    );
    assert!(
        found.contains(&format!("{path}:2: CRLF line end; the repository uses LF")),
        "{found:?}"
    );
    assert!(
        found.contains(&format!("{path}:3: no final newline")),
        "{found:?}"
    );
}

#[test]
fn markdown_keeps_trailing_spaces_and_binaries_and_reference_are_skipped() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write(
        "documentation/break.md",
        "**Status:** live\n\nA line break  \nhere.\n",
    );
    fixture.write("assets/sound.wav", "RIFF \r\n");
    fixture.write("reference/tfar_sample/ts/src/crlf.cpp", "int a;\r\n");
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn ignored_files_are_not_checked() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write("target/debug/output.txt", "trailing \n");
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn a_folder_outside_any_git_repository_could_not_run() {
    let fixture = FixtureTree::empty();
    std::fs::remove_dir_all(fixture.root().join(".git")).unwrap();
    let inside_a_repository = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(fixture.root())
        .output()
        .unwrap()
        .status
        .success();
    if !inside_a_repository {
        assert!(check(&fixture.repository()).is_err());
    }
}
