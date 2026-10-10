//! Unit tests for the plugin's port choice.

use super::*;

fn listen_ports(config: &ServerConfig) -> Vec<u16> {
    config.listen_ports().collect()
}

#[test]
fn without_an_override_the_plugin_listens_on_both_spec_ports() {
    let config = server_config(Flavour::V0_9, None);
    assert_eq!(config.port, 47810);
    assert_eq!(config.debug_port, Some(47811));
    assert_eq!(listen_ports(&config), [47810, 47811]);
}

#[test]
fn a_valid_override_listens_on_that_one_port_alone() {
    let config = server_config(Flavour::V0_9, Some("50123"));
    assert_eq!(listen_ports(&config), [50123]);
    assert_eq!(config.debug_port, None);
    assert_eq!(
        listen_ports(&server_config(Flavour::V0_9, Some(" 1 "))),
        [1]
    );
}

#[test]
fn an_override_that_is_not_a_port_is_ignored() {
    for value in ["0", "65536", "-1", "port", ""] {
        assert_eq!(
            listen_ports(&server_config(Flavour::V0_9, Some(value))),
            [47810, 47811]
        );
    }
}

#[test]
fn the_server_serves_the_0_9_flavour_without_needcfg() {
    for override_value in [None, Some("50123")] {
        let config = server_config(Flavour::V0_9, override_value);
        assert_eq!(config.flavour, Flavour::V0_9);
        assert!(!config.initial_needcfg);
    }
}

#[test]
fn the_variable_name_is_the_documented_one() {
    assert_eq!(PORT_OVERRIDE_VARIABLE, "TFAR_GAME_LINK_PORT");
}
