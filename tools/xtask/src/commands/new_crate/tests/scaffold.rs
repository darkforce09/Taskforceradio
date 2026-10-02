//! Tests for `new-crate`: the scaffold passes the gates and refuses illegal requests.

use super::*;
use crate::commands::verify::gate::Gate;
use crate::fixture_tree::{FixtureTree, described};

fn request(category: &str, name: &str, depends_on: &[&str]) -> NewCrateRequest {
    NewCrateRequest {
        category: category.to_owned(),
        name: name.to_owned(),
        summary: Some("Units and conversions shared by the geometry".to_owned()),
        depends_on: depends_on.iter().map(|name| (*name).to_owned()).collect(),
    }
}

#[test]
fn a_scaffolded_crate_passes_every_gate_and_gets_its_computed_tier() {
    let fixture = FixtureTree::sample_workspace();
    let spec = run(
        &fixture.repository(),
        &request("foundation", "tfar_units", &["tfar_geometry"]),
    )
    .unwrap();
    assert_eq!(spec.tier, 1);
    assert_eq!(spec.folder(), "crates/foundation/tfar_units");
    let manifest = fixture.read("Cargo.toml");
    let geometry = manifest.find("tfar_geometry = ").unwrap();
    let units = manifest
        .find("tfar_units = { path = \"crates/foundation/tfar_units\" }")
        .unwrap();
    let version_banner = manifest.find("# crates/version_0_9:").unwrap();
    assert!(geometry < units && units < version_banner, "{manifest}");
    assert!(
        fixture
            .read("crates/foundation/README.md")
            .contains("└── tfar_units/")
    );
    for gate in Gate::ALL {
        assert_eq!(
            described(&fixture.violations(gate)),
            Vec::<String>::new(),
            "{}",
            gate.name()
        );
    }
}

#[test]
fn a_scaffolded_app_passes_every_gate() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write(
        "apps/README.md",
        "# Apps\n\nThe sample apps.\n\n## Contents\n\n```text\napps/\n```\n\n## How it works\n\nOne app per host.\n\n\
         ## Getting started\n\nNothing to run.\n\n## Boundaries\n\n- Depends on: nothing.\n- Used by: nothing.\n- Rules: none.\n",
    );
    fixture.replace(
        "Cargo.toml",
        "members = [\"crates/*/*\"]",
        "members = [\"apps/*\", \"crates/*/*\"]",
    );
    let spec = run(
        &fixture.repository(),
        &request("apps", "tfar_extension_0_9", &["tfar_geometry"]),
    )
    .unwrap();
    assert_eq!(spec.tier, 1);
    for gate in Gate::ALL {
        assert_eq!(
            described(&fixture.violations(gate)),
            Vec::<String>::new(),
            "{}",
            gate.name()
        );
    }
}

#[test]
fn an_illegal_edge_is_refused_before_anything_is_written() {
    let fixture = FixtureTree::sample_workspace();
    let before = fixture.read("Cargo.toml");
    let refused = run(
        &fixture.repository(),
        &request("version_0_9", "tfar_radio_0_9", &["tfar_game_commands_1_0"]),
    );
    assert!(
        refused
            .unwrap_err()
            .to_string()
            .contains("never depend on each other")
    );
    assert_eq!(fixture.read("Cargo.toml"), before);
    assert!(
        !fixture
            .root()
            .join("crates/version_0_9/tfar_radio_0_9")
            .exists()
    );
}

#[test]
fn bad_names_and_categories_are_refused() {
    let fixture = FixtureTree::sample_workspace();
    let repository = fixture.repository();
    assert!(plan(&repository, &request("foundation", "Units", &[])).is_err());
    assert!(plan(&repository, &request("foundation", "units", &[])).is_err());
    assert!(plan(&repository, &request("version_1_0", "tfar_radio", &[])).is_err());
    assert!(plan(&repository, &request("nowhere", "tfar_x", &[])).is_err());
    assert!(plan(&repository, &request("foundation", "tfar_geometry", &[])).is_err());
    assert_eq!(normalised_category("crates/audio").unwrap(), "crates/audio");
    assert_eq!(normalised_category("tools").unwrap(), "tools");
}
