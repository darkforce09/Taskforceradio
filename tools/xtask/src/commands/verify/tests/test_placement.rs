//! Fixture tests for the `test-placement` gate.

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
fn an_inline_test_module_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        SAMPLE_MODULE,
        "#[cfg(test)]\n#[path = \"tests/sample.rs\"]\nmod tests;\n",
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
        SAMPLE_MODULE,
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
    fixture.replace(SAMPLE_MODULE, "#[path = \"tests/sample.rs\"]\n", "");
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
