//! Unit tests for the exports module of `tfar_extension_1_0`: the version string and the port choice.

use super::*;

#[test]
fn the_version_names_the_crate_and_its_version() {
    assert_eq!(
        EXTENSION_VERSION,
        format!("tfar_extension_1_0 {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn the_client_speaks_the_1_0_flavour_on_the_1_0_ports() {
    assert_eq!(FLAVOUR, Flavour::V1_0);
    assert_eq!(client_config(false).flavour, Flavour::V1_0);
    assert_eq!(client_config(false).port, 47820);
    assert_eq!(client_config(true).port, 47821);
}

#[test]
fn the_debug_switch_is_the_original_one() {
    assert_eq!(DEBUG_SWITCH, "-tfdebug");
}

#[test]
fn the_focus_is_read_at_most_every_250_ms() {
    assert_eq!(
        crate::focus_watch::FOCUS_READ_INTERVAL,
        core::time::Duration::from_millis(250)
    );
}
