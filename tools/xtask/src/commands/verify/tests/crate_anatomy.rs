//! Fixture tests for the `crate-anatomy` gate.

use super::*;
use crate::fixture_tree::{FixtureTree, described};

const GEOMETRY: &str = "crates/foundation/tfar_geometry";

fn found_after(change: impl FnOnce(&FixtureTree)) -> Vec<String> {
    let fixture = FixtureTree::sample_workspace();
    change(&fixture);
    described(&check(&fixture.repository()).unwrap())
}

#[test]
fn the_sample_workspace_holds() {
    assert_eq!(found_after(|_| {}), Vec::<String>::new());
}

#[test]
fn a_missing_prelude_fails() {
    let found = found_after(|fixture| fixture.remove(&format!("{GEOMETRY}/src/prelude.rs")));
    assert!(
        found
            .iter()
            .any(|line| line.contains("needs `src/prelude.rs`")),
        "{found:?}"
    );
}

#[test]
fn an_error_type_without_thiserror_fails() {
    let found = found_after(|fixture| {
        fixture.write(
            &format!("{GEOMETRY}/src/error.rs"),
            "//! Errors.\n\npub enum Error {}\n",
        );
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("must derive `thiserror::Error`")),
        "{found:?}"
    );
}

#[test]
fn a_function_in_lib_rs_fails() {
    let found = found_after(|fixture| {
        fixture.replace(
            &format!("{GEOMETRY}/src/lib.rs"),
            "pub mod prelude;\n",
            "pub mod prelude;\n\npub fn area() -> u32 {\n    1\n}\n",
        );
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("holds only the header")),
        "{found:?}"
    );
}

#[test]
fn an_inline_module_in_lib_rs_fails() {
    let found = found_after(|fixture| {
        fixture.replace(
            &format!("{GEOMETRY}/src/lib.rs"),
            "pub mod prelude;\n",
            "pub mod prelude;\nmod inline {}\n",
        );
    });
    assert!(
        found.iter().any(|line| line.contains("never inline")),
        "{found:?}"
    );
}

#[test]
fn a_lib_rs_over_80_lines_fails() {
    let found = found_after(|fixture| {
        let padding = "\n".repeat(80);
        fixture.replace(
            &format!("{GEOMETRY}/src/lib.rs"),
            "pub mod prelude;\n",
            &format!("pub mod prelude;\n{padding}"),
        );
    });
    assert!(
        found.iter().any(|line| line.contains("the limit is 80")),
        "{found:?}"
    );
}

#[test]
fn a_workspace_crate_re_exported_from_lib_rs_fails() {
    let found = found_after(|fixture| {
        fixture.replace(
            "crates/version_0_9/tfar_game_commands_0_9/src/lib.rs",
            "pub mod prelude;\n",
            "pub mod prelude;\npub use tfar_geometry::CRATE_ROLE as GEOMETRY_ROLE;\n",
        );
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("re-export it from `prelude.rs` only")),
        "{found:?}"
    );
}

#[test]
fn a_dependency_not_from_the_workspace_fails() {
    let found = found_after(|fixture| {
        fixture.replace(
            &format!("{GEOMETRY}/Cargo.toml"),
            "thiserror.workspace = true",
            "thiserror = \"2\"",
        );
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("must be `thiserror.workspace = true`")),
        "{found:?}"
    );
}

#[test]
fn a_package_field_not_from_the_workspace_fails() {
    let found = found_after(|fixture| {
        fixture.replace(
            &format!("{GEOMETRY}/Cargo.toml"),
            "edition.workspace = true",
            "edition = \"2024\"",
        );
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("`edition` must come from the workspace")),
        "{found:?}"
    );
}

#[test]
fn missing_workspace_lints_fail() {
    let found = found_after(|fixture| {
        fixture.replace(
            &format!("{GEOMETRY}/Cargo.toml"),
            "[lints]\nworkspace = true\n",
            "",
        );
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("needs `[lints] workspace = true`")),
        "{found:?}"
    );
}

#[test]
fn a_feature_outside_the_allowed_names_fails_and_test_fixtures_holds() {
    let found = found_after(|fixture| {
        fixture.replace(
            &format!("{GEOMETRY}/Cargo.toml"),
            "[lints]",
            "[features]\ntest_fixtures = []\nfast = []\n\n[lints]",
        );
    });
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("feature `fast` is not allowed"),
        "{found:?}"
    );
}

#[test]
fn the_tracy_feature_outside_the_plugin_apps_fails() {
    let found = found_after(|fixture| {
        fixture.replace(
            &format!("{GEOMETRY}/Cargo.toml"),
            "[lints]",
            "[features]\ntracy = []\n\n[lints]",
        );
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("feature `tracy` is not allowed")),
        "{found:?}"
    );
}

#[test]
fn anyhow_in_a_library_fails() {
    let found = found_after(|fixture| {
        fixture.replace(
            &format!("{GEOMETRY}/Cargo.toml"),
            "thiserror.workspace = true\n",
            "thiserror.workspace = true\nanyhow.workspace = true\n",
        );
        fixture.write(
            &format!("{GEOMETRY}/src/area.rs"),
            "//! Area.\n\nfn area() -> anyhow::Result<u32> {\n    Ok(1)\n}\n",
        );
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("a library never uses `anyhow`")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|line| line.contains("src/area.rs:3: library code never uses `anyhow`")),
        "{found:?}"
    );
}

#[test]
fn an_unparsable_manifest_means_the_gate_could_not_run() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write(&format!("{GEOMETRY}/Cargo.toml"), "[package\n");
    assert!(check(&fixture.repository()).is_err());
}
