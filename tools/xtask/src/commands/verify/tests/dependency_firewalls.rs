//! Fixture tests for the `dependency-firewalls` gate.

use super::*;
use crate::fixture_tree::{FixtureTree, described};

const GEOMETRY: &str = "crates/foundation/tfar_geometry/Cargo.toml";

#[test]
fn the_sample_workspace_holds() {
    let fixture = FixtureTree::sample_workspace();
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn a_firewalled_crate_in_a_crate_not_allowed_to_use_it_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        GEOMETRY,
        "thiserror.workspace = true\n",
        "thiserror.workspace = true\nrusqlite.workspace = true\n",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("`rusqlite` is firewalled: only tfar_ts3_settings_db"),
        "{found:?}"
    );
}

#[test]
fn anyhow_in_a_library_crate_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        GEOMETRY,
        "thiserror.workspace = true\n",
        "thiserror.workspace = true\nanyhow.workspace = true\n",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("`anyhow` is firewalled: only the tools crates")),
        "{found:?}"
    );
}

#[test]
fn a_firewalled_crate_in_its_allowed_crate_holds() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write_crate("crates/teamspeak", "tfar_ts3_settings_db", 0, &[]);
    fixture.replace(
        "crates/teamspeak/tfar_ts3_settings_db/Cargo.toml",
        "thiserror.workspace = true\n",
        "thiserror.workspace = true\nrusqlite.workspace = true\n",
    );
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}
