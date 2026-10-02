//! Fixture tests for the `module-headers` gate.

use super::*;
use crate::fixture_tree::{FixtureTree, described};

const GEOMETRY_LIB: &str = "crates/foundation/tfar_geometry/src/lib.rs";

#[test]
fn the_sample_workspace_holds() {
    let fixture = FixtureTree::sample_workspace();
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn a_crate_root_without_the_header_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write(
        GEOMETRY_LIB,
        "//! Geometry.\n\nmod crate_role;\nmod error;\npub mod prelude;\n",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("a crate root needs the four-point")),
        "{found:?}"
    );
}

#[test]
fn a_production_file_of_80_lines_needs_the_header_and_79_does_not() {
    let fixture = FixtureTree::sample_workspace();
    let body = "//! Long.\n".to_owned() + &"// line\n".repeat(78);
    fixture.write("crates/foundation/tfar_geometry/src/long.rs", &body);
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
    fixture.write(
        "crates/foundation/tfar_geometry/src/long.rs",
        &(body + "// line\n"),
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("a file of 80 lines or more")),
        "{found:?}"
    );
}

#[test]
fn the_labels_must_come_in_order_each_in_its_own_paragraph() {
    let whole = "//! Summary.\n//!\n//! **Role:** r\n//!\n//! **Position:** p\n//!\n//! **Signals and state:** s\n//!\n//! **Invariants:** i\n";
    assert_eq!(header_problem(whole), None);
    let swapped = whole.replace("**Role:** r", "**Position:** x").replacen(
        "**Position:** p",
        "**Role:** r",
        1,
    );
    assert!(header_problem(&swapped).is_some());
    let glued = whole.replace("//! **Role:** r\n//!\n", "//! **Role:** r\n");
    assert!(header_problem(&glued).unwrap().1.contains("own paragraph"));
    let old_label = whole.replace("**Signals and state:**", "**Signals:**");
    assert!(
        header_problem(&old_label)
            .unwrap()
            .1
            .contains("Signals and state")
    );
    assert_eq!(header_problem("mod a;\n").unwrap().0, 1);
}
