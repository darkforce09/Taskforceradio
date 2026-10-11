//! Tests for the layout tables: they say what the laws say.

use super::*;

#[test]
fn foundation_depends_only_on_foundation() {
    assert!(category_allows(FOUNDATION, FOUNDATION));
    assert!(!category_allows(FOUNDATION, "crates/audio"));
}

#[test]
fn the_version_categories_never_meet() {
    assert!(
        edge_violation(
            "tfar_a_0_9",
            VERSION_0_9,
            "tfar_b_1_0",
            VERSION_1_0,
            EdgeTable::Development
        )
        .is_some()
    );
    assert!(
        edge_violation(
            "tfar_plugin_1_0",
            VERSION_1_0,
            "tfar_b_0_9",
            VERSION_0_9,
            EdgeTable::NormalOrBuild
        )
        .is_some()
    );
    assert!(
        edge_violation(
            "tfar_a_0_9",
            VERSION_0_9,
            "tfar_b_0_9",
            VERSION_0_9,
            EdgeTable::NormalOrBuild
        )
        .is_none()
    );
}

#[test]
fn version_crates_use_only_the_protocol_from_game_link() {
    assert!(
        edge_violation(
            "tfar_a_1_0",
            VERSION_1_0,
            GAME_LINK_CRATE,
            GAME_LINK,
            EdgeTable::NormalOrBuild
        )
        .is_none()
    );
    assert!(
        edge_violation(
            "tfar_a_1_0",
            VERSION_1_0,
            "tfar_arma_extension_abi",
            GAME_LINK,
            EdgeTable::NormalOrBuild
        )
        .is_some()
    );
}

#[test]
fn extension_apps_use_only_foundation_and_game_link() {
    assert!(app_allows("tfar_extension_0_9", GAME_LINK));
    assert!(app_allows("tfar_extension_1_0", FOUNDATION));
    assert!(!app_allows("tfar_extension_1_0", "crates/audio"));
    assert!(!app_allows("tfar_extension_1_0", VERSION_1_0));
    assert!(!app_allows("tfar_a_1_0", FOUNDATION));
    assert!(
        edge_violation(
            "tfar_extension_0_9",
            VERSION_0_9,
            "tfar_game_link",
            GAME_LINK,
            EdgeTable::NormalOrBuild
        )
        .is_none()
    );
    assert!(
        edge_violation(
            "tfar_extension_0_9",
            VERSION_0_9,
            "tfar_plugin_logic_0_9",
            VERSION_0_9,
            EdgeTable::NormalOrBuild
        )
        .is_some()
    );
}

#[test]
fn plugin_apps_use_any_game_link_crate_and_their_own_version() {
    assert!(
        edge_violation(
            "tfar_plugin_0_9",
            VERSION_0_9,
            "tfar_game_link",
            GAME_LINK,
            EdgeTable::NormalOrBuild
        )
        .is_none()
    );
    assert!(
        edge_violation(
            "tfar_plugin_0_9",
            VERSION_0_9,
            "tfar_plugin_logic_0_9",
            VERSION_0_9,
            EdgeTable::NormalOrBuild
        )
        .is_none()
    );
}

#[test]
fn a_version_library_never_depends_on_its_own_deliverables() {
    assert!(
        edge_violation(
            "tfar_plugin_logic_1_0",
            VERSION_1_0,
            "tfar_plugin_1_0",
            VERSION_1_0,
            EdgeTable::NormalOrBuild
        )
        .is_some()
    );
    assert!(
        edge_violation(
            "tfar_plugin_logic_0_9",
            VERSION_0_9,
            "tfar_extension_0_9",
            VERSION_0_9,
            EdgeTable::Development
        )
        .is_some()
    );
}

#[test]
fn nothing_depends_on_an_app_and_tools_use_crates() {
    assert!(
        edge_violation(
            "tfar_ts3_mock",
            TOOLS,
            "tfar_plugin_1_0",
            VERSION_1_0,
            EdgeTable::Development
        )
        .is_some()
    );
    assert!(
        edge_violation(
            "tfar_plugin_1_0",
            VERSION_1_0,
            "tfar_extension_1_0",
            VERSION_1_0,
            EdgeTable::NormalOrBuild
        )
        .is_some()
    );
    assert!(
        edge_violation(
            "tfar_ts3_mock",
            TOOLS,
            GAME_LINK_CRATE,
            GAME_LINK,
            EdgeTable::NormalOrBuild
        )
        .is_none()
    );
}

#[test]
fn every_firewalled_crate_appears_once() {
    let mut names: Vec<&str> = FIREWALLS.iter().map(|(name, _)| *name).collect();
    let count = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), count);
}

#[test]
fn header_only_folders_admit_only_dot_h() {
    assert!(c_file_allowed(
        "crates/teamspeak/tfar_ts3_host/sdk_headers/ts3_functions.h",
        "h"
    ));
    assert!(!c_file_allowed(
        "crates/teamspeak/tfar_ts3_host/sdk_headers/shim.c",
        "c"
    ));
    assert!(c_file_allowed(
        "tools/tfar_golden_vectors/oracle/a.cpp",
        "cpp"
    ));
    assert!(!c_file_allowed(
        "crates/teamspeak/tfar_ts3_host/src/a.h",
        "h"
    ));
}

#[test]
fn the_ffi_crates_the_probe_and_the_oracle_may_lift_unsafe() {
    for name in [
        "tfar_ts3_host",
        "tfar_ts3_mock",
        "tfar_extension_probe",
        "tfar_golden_vectors",
    ] {
        assert!(UNSAFE_ALLOWED.contains(&name), "{name}");
    }
    assert!(!UNSAFE_ALLOWED.contains(&"tfar_foundation"));
}
