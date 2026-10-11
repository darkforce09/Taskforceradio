//! Unit tests for the plugin's port choice.

use super::*;

fn ports(config: &ServerConfig) -> Vec<u16> {
    config.listen_ports().collect()
}

#[test]
fn without_an_override_the_plugin_listens_on_both_spec_ports() {
    let config = server_config(Flavour::V1_0, None);
    assert_eq!(config.port, 47820);
    assert_eq!(config.debug_port, Some(47821));
    assert_eq!(ports(&config), [47820, 47821]);
}

#[test]
fn a_valid_override_is_the_one_port_listened_on() {
    assert_eq!(ports(&server_config(Flavour::V1_0, Some("50123"))), [50123]);
    assert_eq!(ports(&server_config(Flavour::V1_0, Some(" 1 "))), [1]);
}

#[test]
fn an_override_that_is_not_a_port_keeps_both_spec_ports() {
    for value in ["0", "65536", "-1", "port", ""] {
        assert_eq!(
            ports(&server_config(Flavour::V1_0, Some(value))),
            [47820, 47821],
            "{value:?}"
        );
    }
}

#[test]
fn the_server_serves_the_1_0_flavour_starting_with_needcfg() {
    for config in [
        server_config(Flavour::V1_0, None),
        server_config(Flavour::V1_0, Some("50123")),
    ] {
        assert_eq!(config.flavour, Flavour::V1_0);
        assert!(config.initial_needcfg);
    }
}
