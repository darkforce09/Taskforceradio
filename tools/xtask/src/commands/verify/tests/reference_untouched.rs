//! Fixture tests for the `reference-untouched` gate.

use super::*;
use crate::fixture_tree::{FixtureTree, described};

const SOURCE: &str = "reference/tfar_sample/ts/src/plugin.cpp";

#[test]
fn the_sample_workspace_holds() {
    let fixture = FixtureTree::sample_workspace();
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn a_changed_byte_under_reference_fails() {
    let fixture = FixtureTree::sample_workspace();
    let mut text = fixture.read(SOURCE);
    text.push(' ');
    fixture.write(SOURCE, &text);
    let found = described(&check(&fixture.repository()).unwrap());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].starts_with(&format!("{SOURCE}:1: changed: sha256")),
        "{found:?}"
    );
}

#[test]
fn added_and_removed_files_are_reported() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write("reference/tfar_sample/new.txt", "new\n");
    fixture.remove(SOURCE);
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found.contains(&"reference/tfar_sample/new.txt:1: added: not in the manifest".to_owned()),
        "{found:?}"
    );
    assert!(
        found.contains(&format!(
            "{SOURCE}:1: removed: listed in the manifest but missing"
        )),
        "{found:?}"
    );
}

#[test]
fn the_readme_the_manifest_and_the_1_0_git_folder_are_left_out() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write("reference/README.md", "# Reference, reworded\n");
    fixture.write("reference/tfar_1_0/.git/HEAD", "ref: refs/heads/master\n");
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn a_missing_manifest_means_the_gate_could_not_run() {
    let fixture = FixtureTree::sample_workspace();
    fixture.remove("reference/MANIFEST.sha256");
    assert!(check(&fixture.repository()).is_err());
}
