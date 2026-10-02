//! Fixture tests for the `test-placement` gate.

use super::*;
use crate::fixture_tree::{FixtureTree, described};

const ROLE: &str = "crates/foundation/tfar_geometry/src/crate_role.rs";

#[test]
fn the_sample_workspace_holds() {
    let fixture = FixtureTree::sample_workspace();
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn an_inline_test_module_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        ROLE,
        "#[cfg(test)]\n#[path = \"tests/crate_role.rs\"]\nmod tests;\n",
        "#[cfg(test)]\nmod tests {\n    #[test]\n    fn holds() {}\n}\n",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("inline test module `mod tests {`")),
        "{found:?}"
    );
}

#[test]
fn a_test_function_in_a_production_file_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        ROLE,
        "mod tests;\n",
        "mod tests;\n\n#[test]\nfn holds() {}\n",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("a `#[test]` function in a production file")),
        "{found:?}"
    );
}

#[test]
fn a_test_module_without_a_path_attribute_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(ROLE, "#[path = \"tests/crate_role.rs\"]\n", "");
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("must attach with `#[path")),
        "{found:?}"
    );
}

#[test]
fn inline_modules_inside_test_files_and_strings_are_fine() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write(
        "crates/foundation/tfar_geometry/src/tests/helpers.rs",
        "mod tests {\n}\n",
    );
    fixture.write(
        "crates/foundation/tfar_geometry/src/text.rs",
        "//! Text.\n\nconst TEXT: &str = \"mod tests {\";\n",
    );
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}
