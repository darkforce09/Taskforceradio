//! Fixture tests for the `history-residue` gate.

use super::*;
use crate::fixture_tree::{FixtureTree, described};

const SAMPLE_MODULE: &str = "crates/foundation/tfar_geometry/src/sample.rs";

#[test]
fn the_sample_workspace_holds() {
    let fixture = FixtureTree::sample_workspace();
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn a_bug_catalogue_id_in_a_comment_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        SAMPLE_MODULE,
        "/// A sample value.",
        "/// A sample value (F-012).",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("`F-012` is a program id"), "{found:?}");
}

#[test]
fn a_decision_id_in_a_crate_readme_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write(
        "crates/foundation/tfar_geometry/README.md",
        "# Geometry\n\nDormant again after a game ends (decision D25).\n",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert_eq!(
        found,
        [
            "crates/foundation/tfar_geometry/README.md:3: `D25` is a program id: say what the code does now; ids live in documentation/"
        ]
    );
}

#[test]
fn ids_in_documentation_are_allowed() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write(
        "documentation/bug_fixes.md",
        "# Bug fixes\n\nF-001 and D3, landed in M4.\n",
    );
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn only_whole_words_are_ids() {
    for id in ["F-001", "OF-029", "D3", "D25", "M8"] {
        assert!(is_program_id(id), "{id}");
    }
    for word in [
        "F-01", "F-0001", "D", "D123", "M10", "MD5", "Dx", "SF-001", "F-00a",
    ] {
        assert!(!is_program_id(word), "{word}");
    }
}
