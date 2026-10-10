//! Unit tests for the exports module of `tfar_extension_0_9`: the version string and the port choice.

use super::*;

#[test]
fn the_version_names_the_crate_and_its_version() {
    assert_eq!(
        EXTENSION_VERSION,
        format!("tfar_extension_0_9 {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn the_client_speaks_the_0_9_flavour_on_the_0_9_ports() {
    assert_eq!(FLAVOUR, Flavour::V0_9);
    assert_eq!(client_config(false).flavour, Flavour::V0_9);
    assert_eq!(client_config(false).port, 47810);
    assert_eq!(client_config(true).port, 47811);
}

#[test]
fn the_debug_switch_is_the_original_one() {
    assert_eq!(DEBUG_SWITCH, "-tfdebug");
}
