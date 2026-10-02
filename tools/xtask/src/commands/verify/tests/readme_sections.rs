//! Fixture tests for the `readme-sections` gate.

use super::*;
use crate::fixture_tree::{FixtureTree, described};

const GEOMETRY_README: &str = "crates/foundation/tfar_geometry/README.md";

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
fn sections_out_of_order_fail() {
    let found = found_after(|fixture| {
        let text = fixture.read(GEOMETRY_README);
        let (before, configuration) = text
            .split_once("## Configuration\n\nNone: the crate reads no setting.\n\n")
            .unwrap();
        let moved = before.replace(
            "## How it works",
            "## Configuration\n\nNone.\n\n## How it works",
        );
        fixture.write(GEOMETRY_README, &(moved + configuration));
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("`## How it works` is out of order")),
        "{found:?}"
    );
}

#[test]
fn a_heading_outside_the_standard_fails() {
    let found = found_after(|fixture| {
        fixture.replace(
            GEOMETRY_README,
            "## Boundaries",
            "## Notes\n\nNone.\n\n## Boundaries",
        );
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("`## Notes` is not a README heading")),
        "{found:?}"
    );
}

#[test]
fn a_section_of_another_kind_fails() {
    let found = found_after(|fixture| {
        fixture.replace(
            GEOMETRY_README,
            "## Boundaries",
            "## Exports\n\nNone.\n\n## Boundaries",
        );
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("`## Exports` is not a section of the crate root kind")),
        "{found:?}"
    );
}

#[test]
fn a_missing_kind_section_fails() {
    let found = found_after(|fixture| {
        fixture.replace(
            GEOMETRY_README,
            "## Configuration\n\nNone: the crate reads no setting.\n\n",
            "",
        );
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("needs a `## Configuration` section")),
        "{found:?}"
    );
}

#[test]
fn boundaries_without_its_three_bullets_fails() {
    let found = found_after(|fixture| {
        fixture.replace(
            GEOMETRY_README,
            "- Used by: nothing.\n",
            "- Users: nothing.\n",
        );
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("must start with `- Used by:`")),
        "{found:?}"
    );
    let found =
        found_after(|fixture| fixture.replace(GEOMETRY_README, "- Used by: nothing.\n", ""));
    assert!(
        found
            .iter()
            .any(|line| line.contains("exactly three top-level bullets; found 2")),
        "{found:?}"
    );
}

#[test]
fn a_code_readme_with_a_status_line_or_a_backticked_title_fails() {
    let found = found_after(|fixture| {
        let text = fixture.read(GEOMETRY_README).replacen(
            "# Geometry",
            "**Status:** live\n\n# `tfar_geometry`",
            1,
        );
        fixture.write(GEOMETRY_README, &text);
    });
    assert!(
        found
            .iter()
            .any(|line| line.contains("carries no status line")),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|line| line.contains("no backticks, no path")),
        "{found:?}"
    );
}

#[test]
fn headings_inside_code_fences_are_not_sections() {
    let found = found_after(|fixture| {
        fixture.replace(
            GEOMETRY_README,
            "## Boundaries",
            "## Boundaries\n\n````markdown\n## Notes\n````\n",
        );
    });
    assert_eq!(found, Vec::<String>::new());
}

#[test]
fn a_missing_readme_standard_means_the_gate_could_not_run() {
    let fixture = FixtureTree::sample_workspace();
    fixture.remove("documentation/standards/readme_standard.md");
    assert!(check(&fixture.repository()).is_err());
}

#[test]
fn the_real_readme_standard_parses() {
    let text = include_str!("../../../../../../documentation/standards/readme_standard.md");
    let standard = ReadmeStandard::parse(text).unwrap();
    assert_eq!(
        standard.sections_of("app"),
        ["Getting started", "Configuration", "Exports"]
    );
    assert_eq!(standard.order.first().map(String::as_str), Some("Contents"));
    assert_eq!(
        standard.order.last().map(String::as_str),
        Some("Related documentation")
    );
}
