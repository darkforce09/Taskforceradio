//! Fixture tests for the `crate-tiers` gate: the sample workspace holds; each broken law fails.

use super::*;
use crate::fixture_tree::{FixtureTree, described};

const COMMANDS_0_9: &str = "crates/version_0_9/tfar_game_commands_0_9/Cargo.toml";

#[test]
fn the_sample_workspace_holds() {
    let fixture = FixtureTree::sample_workspace();
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn a_version_0_9_crate_depending_on_a_version_1_0_crate_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        COMMANDS_0_9,
        "tfar_geometry.workspace = true\n",
        "tfar_geometry.workspace = true\ntfar_game_commands_1_0.workspace = true\n",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("never depend on each other")),
        "{found:?}"
    );
}

#[test]
fn a_version_ban_holds_for_dev_dependencies_too() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        COMMANDS_0_9,
        "[lints]",
        "[dev-dependencies]\ntfar_game_commands_1_0.workspace = true\n\n[lints]",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("never depend on each other")),
        "{found:?}"
    );
}

#[test]
fn a_wrong_declared_tier_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(COMMANDS_0_9, "tier = 1", "tier = 3");
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("declared tier 3, computed tier 1")),
        "{found:?}"
    );
}

#[test]
fn a_foundation_crate_depending_on_a_version_crate_breaks_the_matrix() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        "crates/foundation/tfar_geometry/Cargo.toml",
        "thiserror.workspace = true\n",
        "thiserror.workspace = true\ntfar_game_commands_0_9.workspace = true\n",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("not in the category edge matrix")),
        "{found:?}"
    );
}

#[test]
fn a_missing_layout_table_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        COMMANDS_0_9,
        "[package.metadata.layout]",
        "[package.metadata.other]",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("no [package.metadata.layout]")),
        "{found:?}"
    );
}

#[test]
fn a_crate_in_the_wrong_category_folder_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        COMMANDS_0_9,
        "category = \"crates/version_0_9\"",
        "category = \"crates/audio\"",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found.iter().any(|line| line.contains("category plus name")),
        "{found:?}"
    );
}

#[test]
fn a_manifest_outside_the_workspace_members_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        "Cargo.toml",
        "members = [\"crates/*/*\"]",
        "members = [\"crates/*/*\"]\nexclude = [\"tools/stray\"]",
    );
    fixture.write("tools/stray/Cargo.toml", "[package]\nname = \"stray\"\n");
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found.iter().any(|line| line
            .contains("tools/stray/Cargo.toml:1: this manifest is not a workspace member")),
        "{found:?}"
    );
}

#[test]
fn a_broken_root_manifest_means_the_gate_could_not_run() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write("Cargo.toml", "[workspace\n");
    assert!(check(&fixture.repository()).is_err());
}
