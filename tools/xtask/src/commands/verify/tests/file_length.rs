//! Fixture tests for the `file-length` gate.

use super::*;
use crate::fixture_tree::{FixtureTree, described};

fn lines(count: usize) -> String {
    "// line\n".repeat(count)
}

#[test]
fn the_sample_workspace_holds() {
    let fixture = FixtureTree::sample_workspace();
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn a_production_file_of_501_lines_fails_and_500_holds() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write("crates/foundation/tfar_geometry/src/long.rs", &lines(500));
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
    fixture.write("crates/foundation/tfar_geometry/src/long.rs", &lines(501));
    let found = described(&check(&fixture.repository()).unwrap());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].starts_with("crates/foundation/tfar_geometry/src/long.rs:501: 501 lines"),
        "{found:?}"
    );
}

#[test]
fn a_test_file_may_run_to_1000_lines_but_not_1001() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write(
        "crates/foundation/tfar_geometry/src/tests/long.rs",
        &lines(1000),
    );
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
    fixture.write(
        "crates/foundation/tfar_geometry/src/tests/long.rs",
        &lines(1001),
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("a test file stays within 1000")),
        "{found:?}"
    );
}

#[test]
fn a_tree_without_source_roots_could_not_run() {
    let fixture = FixtureTree::empty();
    fixture.write("README.md", "# Nothing\n");
    assert!(check(&fixture.repository()).is_err());
}
