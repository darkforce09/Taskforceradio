//! Unit tests for the exports module of `tfar_extension_1_0`: the version string and the port choice.

use super::*;

#[test]
fn the_client_speaks_the_1_0_flavour_on_the_1_0_ports() {
    assert_eq!(client_config(false).flavour, Flavour::V1_0);
    assert_eq!(client_config(false).port, 47820);
    assert_eq!(client_config(true).port, 47821);
}

#[test]
fn the_focus_is_read_at_most_every_250_ms() {
    assert_eq!(
        crate::focus_watch::FOCUS_READ_INTERVAL,
        core::time::Duration::from_millis(250)
    );
}
