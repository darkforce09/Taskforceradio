//! Fixture tests for the `language-bans` gate.

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
fn a_python_file_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write("tools/helper.py", "print('no')\n");
    let found = described(&check(&fixture.repository()).unwrap());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].starts_with("tools/helper.py:1: no Python, shell"),
        "{found:?}"
    );
}

#[test]
fn shell_node_and_makefiles_fail() {
    let fixture = FixtureTree::sample_workspace();
    for file in ["run.sh", "run.bash", "tools/a.js", "tools/b.ts", "Makefile"] {
        fixture.write(file, "x\n");
    }
    assert_eq!(check(&fixture.repository()).unwrap().len(), 5);
}

#[test]
fn c_sources_hold_only_in_the_oracle_and_shim_folders() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write("tools/tfar_golden_vectors/oracle/F3DAudio.c", "int a;\n");
    fixture.write("tools/tfar_golden_vectors/shim/shim.cpp", "int b;\n");
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
    fixture.write("crates/foundation/tfar_geometry/src/fast.c", "int c;\n");
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("fast.c:1: C and C++ sources live only")),
        "{found:?}"
    );
}

#[test]
fn files_under_reference_are_ignored() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write(
        "reference/tfar_sample/tools/build.py",
        "print('original')\n",
    );
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}
