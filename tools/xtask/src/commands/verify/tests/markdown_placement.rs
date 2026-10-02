//! Fixture tests for the `markdown-placement` gate.

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
fn a_markdown_file_in_a_code_tree_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write("crates/foundation/notes.md", "# Notes\n");
    let found = described(&check(&fixture.repository()).unwrap());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].starts_with("crates/foundation/notes.md:1:"),
        "{found:?}"
    );
}

#[test]
fn credits_under_assets_and_markdown_under_reference_hold() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write("assets/hrir/CREDITS.md", "# Credits\n");
    fixture.write("reference/tfar_sample/README_EN.md", "# Original\n");
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn a_live_document_over_500_lines_fails_but_a_frozen_record_may() {
    let fixture = FixtureTree::sample_workspace();
    let body = "line\n".repeat(500);
    fixture.write(
        "documentation/long.md",
        &format!("**Status:** live\n{body}"),
    );
    fixture.write(
        "documentation/frozen.md",
        &format!("**Status:** frozen record (2026-10-02)\n{body}"),
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].starts_with("documentation/long.md:501:"),
        "{found:?}"
    );
}
