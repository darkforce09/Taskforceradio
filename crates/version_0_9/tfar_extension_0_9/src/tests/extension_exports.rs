//! Unit tests for the exports module of `tfar_extension_0_9`: the version string and the port choice.

use super::*;

#[test]
fn the_client_speaks_the_0_9_flavour_on_the_0_9_ports() {
    assert_eq!(client_config(false).flavour, Flavour::V0_9);
    assert_eq!(client_config(false).port, 47810);
    assert_eq!(client_config(true).port, 47811);
}
