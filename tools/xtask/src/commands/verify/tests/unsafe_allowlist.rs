//! Fixture tests for the `unsafe-allowlist` gate.

use super::*;
use crate::fixture_tree::{FixtureTree, described};

const GEOMETRY_LIB: &str = "crates/foundation/tfar_geometry/src/lib.rs";
const HOST_LIB: &str = "crates/teamspeak/tfar_ts3_host/src/lib.rs";
const LIFT: &str = "#![allow(unsafe_code)]\n";

fn with_host_crate() -> FixtureTree {
    let fixture = FixtureTree::sample_workspace();
    fixture.write_crate("crates/teamspeak", "tfar_ts3_host", 0, &[]);
    fixture
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
fn an_allow_listed_crate_root_with_a_reason_comment_holds() {
    let fixture = with_host_crate();
    fixture.replace(
        HOST_LIB,
        "mod crate_role;",
        &format!("// The TeamSpeak C ABI lives here.\n{LIFT}\nmod crate_role;"),
    );
    fixture.write(
        "crates/teamspeak/tfar_ts3_host/src/crate_role.rs",
        "//! Role.\n\nfn read(pointer: *const u8) -> u8 {\n    // SAFETY: the caller passes a valid pointer.\n    unsafe { *pointer }\n}\n",
    );
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn a_lift_without_a_reason_comment_fails() {
    let fixture = with_host_crate();
    fixture.replace(
        HOST_LIB,
        "mod crate_role;",
        &format!("{LIFT}\nmod crate_role;"),
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("needs a reason comment")),
        "{found:?}"
    );
}

#[test]
fn a_lift_in_a_crate_off_the_allow_list_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        GEOMETRY_LIB,
        "mod crate_role;",
        &format!("// A reason.\n{LIFT}\nmod crate_role;"),
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("`tfar_geometry` may not lift `unsafe_code`")),
        "{found:?}"
    );
}

#[test]
fn a_multi_line_lift_off_the_allow_list_fails() {
    let fixture = FixtureTree::sample_workspace();
    fixture.replace(
        GEOMETRY_LIB,
        "mod crate_role;",
        "#![allow(\n    unsafe_code,\n    reason = \"glue\"\n)]\n\nmod crate_role;",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found.iter().any(|line| line.contains("may not lift")),
        "{found:?}"
    );
}

#[test]
fn a_lift_outside_the_crate_root_fails() {
    let fixture = with_host_crate();
    fixture.write(
        "crates/teamspeak/tfar_ts3_host/src/crate_role.rs",
        "//! Role.\n\n// A reason.\n#[allow(unsafe_code)]\nfn nothing() {}\n",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert!(
        found
            .iter()
            .any(|line| line.contains("only with `#![allow(unsafe_code)]` at the crate root")),
        "{found:?}"
    );
}

#[test]
fn an_unsafe_block_without_a_safety_comment_fails() {
    let fixture = with_host_crate();
    fixture.write(
        "crates/teamspeak/tfar_ts3_host/src/crate_role.rs",
        "//! Role.\n\nfn read(pointer: *const u8) -> u8 {\n    // Reads the byte.\n    unsafe { *pointer }\n}\n",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert_eq!(found, vec!["crates/teamspeak/tfar_ts3_host/src/crate_role.rs:5: an `unsafe` block needs a `// SAFETY:` comment directly above it".to_owned()]);
}

#[test]
fn unsafe_words_inside_strings_and_comments_are_ignored() {
    let fixture = FixtureTree::sample_workspace();
    fixture.write(
        "crates/foundation/tfar_geometry/src/crate_role.rs",
        "//! Role: an unsafe { block in a comment.\n\nconst TEXT: &str = \"#![allow(unsafe_code)] unsafe { }\";\n",
    );
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn a_safety_comment_above_the_statement_holding_the_block_counts() {
    let fixture = with_host_crate();
    fixture.write(
        "crates/teamspeak/tfar_ts3_host/src/crate_role.rs",
        "//! Role.\n\nfn read(pointer: *const u8) -> u8 {\n    // SAFETY: the caller passes a valid pointer.\n    let value =\n        unsafe { *pointer };\n    value\n}\n",
    );
    assert_eq!(
        described(&check(&fixture.repository()).unwrap()),
        Vec::<String>::new()
    );
}

#[test]
fn a_safety_comment_above_an_earlier_statement_does_not_count() {
    let fixture = with_host_crate();
    fixture.write(
        "crates/teamspeak/tfar_ts3_host/src/crate_role.rs",
        "//! Role.\n\nfn read(pointer: *const u8) -> u8 {\n    // SAFETY: about the line below only.\n    let first = 1;\n    let value =\n        unsafe { *pointer };\n    value + first\n}\n",
    );
    let found = described(&check(&fixture.repository()).unwrap());
    assert_eq!(found.len(), 1, "{found:?}");
}
