//! Fixture tests for the `status-lines` gate.

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
fn a_document_without_a_status_line_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write("documentation/glossary.md", "# Glossary\n");
    let found = described(&check(&fixture.repository()).unwrap());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].starts_with("documentation/glossary.md:1: a document opens with"),
        "{found:?}"
    );
}

#[test]
fn a_code_readme_with_a_status_line_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        "crates/README.md",
        "# Crates",
        "**Status:** live\n\n# Crates",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("crates/README.md:1: a code README")),
        "{found:?}"
    );
}

#[test]
fn the_three_status_lines_are_recognised() {
    assert!(is_status_line("**Status:** live"));
    assert!(is_status_line("**Status:** archived"));
    assert!(is_status_line("**Status:** frozen record (2026-10-02)"));
    assert!(!is_status_line("**Status:** frozen record (2026-10)"));
    assert!(!is_status_line("**Status:** Live"));
    assert!(!is_status_line("Status: live"));
}
