//! Unit tests for the plugin's port choice.

use super::*;

#[test]
fn without_variables_the_plugin_listens_on_the_spec_port() {
    assert_eq!(server_config(Flavour::V1_0, None, None).port, 47820);
}

#[test]
fn the_debug_variable_set_to_one_picks_the_debug_port() {
    assert_eq!(server_config(Flavour::V1_0, Some("1"), None).port, 47821);
}

#[test]
fn any_other_debug_value_keeps_the_spec_port() {
    for value in ["0", "", "true", "yes", " 1"] {
        assert_eq!(server_config(Flavour::V1_0, Some(value), None).port, 47820);
    }
}

#[test]
fn a_valid_override_wins_over_the_debug_variable() {
    assert_eq!(
        server_config(Flavour::V1_0, Some("1"), Some("50123")).port,
        50123
    );
    assert_eq!(server_config(Flavour::V1_0, None, Some(" 1 ")).port, 1);
}

#[test]
fn an_override_that_is_not_a_port_is_ignored() {
    for value in ["0", "65536", "-1", "port", ""] {
        assert_eq!(server_config(Flavour::V1_0, None, Some(value)).port, 47820);
        assert_eq!(
            server_config(Flavour::V1_0, Some("1"), Some(value)).port,
            47821
        );
    }
}

#[test]
fn the_server_serves_the_1_0_flavour_starting_with_needcfg() {
    let config = server_config(Flavour::V1_0, None, Some("50123"));
    assert_eq!(config.flavour, Flavour::V1_0);
    assert!(config.initial_needcfg);
}

#[test]
fn the_variable_names_are_the_documented_ones() {
    assert_eq!(DEBUG_PORT_VARIABLE, "TFAR_GAME_LINK_DEBUG");
    assert_eq!(PORT_OVERRIDE_VARIABLE, "TFAR_GAME_LINK_PORT");
}
