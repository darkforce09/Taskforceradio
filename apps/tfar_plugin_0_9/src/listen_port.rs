//! Which `127.0.0.1` port the plugin's game link listens on: the spec port, the debug port, or a
//! test override, all read from TeamSpeak's environment.

use tfar_game_link_server::prelude::{Flavour, ServerConfig};

#[cfg(test)]
#[path = "tests/listen_port.rs"]
mod tests;

/// The environment variable that, set to `1`, makes the plugin listen on the debug port a
/// `-tfdebug` Arma instance connects to.
pub const DEBUG_PORT_VARIABLE: &str = "TFAR_GAME_LINK_DEBUG";

/// The test-only environment variable that names the port outright (1 to 65535), so test suites
/// that load the plugin never fight over the fixed ports. Any other value is ignored with a
/// warning.
pub const PORT_OVERRIDE_VARIABLE: &str = "TFAR_GAME_LINK_PORT";

/// The server settings for `flavour`, given the values of [`DEBUG_PORT_VARIABLE`] and
/// [`PORT_OVERRIDE_VARIABLE`] (`None` when unset).
///
/// A valid override wins; otherwise a debug value of exactly `1` picks the debug port, and
/// anything else the normal port.
pub fn server_config(
    flavour: Flavour,
    debug_value: Option<&str>,
    override_value: Option<&str>,
) -> ServerConfig {
    match override_value.and_then(parse_port_override) {
        Some(port) => ServerConfig::new(flavour, port),
        None => ServerConfig::for_plugin(flavour, debug_value == Some("1")),
    }
}

/// [`server_config`] over the process environment, logging an override that is not a port.
pub fn server_config_from_environment(flavour: Flavour) -> ServerConfig {
    let debug_value = std::env::var(DEBUG_PORT_VARIABLE).ok();
    let override_value = std::env::var(PORT_OVERRIDE_VARIABLE).ok();
    if let Some(value) = override_value.as_deref()
        && parse_port_override(value).is_none()
    {
        tracing::warn!(
            variable = PORT_OVERRIDE_VARIABLE,
            value,
            "the game link port override is not a port from 1 to 65535; it is ignored"
        );
    }
    server_config(flavour, debug_value.as_deref(), override_value.as_deref())
}

/// The port an override value names, or `None` for anything but a number from 1 to 65535.
fn parse_port_override(value: &str) -> Option<u16> {
    value.trim().parse::<u16>().ok().filter(|port| *port != 0)
}
