//! Which `127.0.0.1` ports the plugin's game link listens on: both spec ports of the flavour (the
//! normal one and the one a `-tfdebug` Arma instance connects to), or a single test override read
//! from TeamSpeak's environment.

use tfar_game_link_server::prelude::{Flavour, ServerConfig};

#[cfg(test)]
#[path = "tests/listen_port.rs"]
mod tests;

/// The test-only environment variable that names one port outright (1 to 65535), so test suites
/// that load the plugin never fight over the fixed ports. With a valid override the plugin listens
/// on that port alone, with no debug port. Any other value is ignored with a warning.
pub const PORT_OVERRIDE_VARIABLE: &str = "TFAR_GAME_LINK_PORT";

/// The server settings for `flavour`, given the value of [`PORT_OVERRIDE_VARIABLE`] (`None` when
/// unset).
///
/// A valid override makes the server listen on that one port; otherwise it listens on both spec
/// ports of `flavour`.
pub fn server_config(flavour: Flavour, override_value: Option<&str>) -> ServerConfig {
    match override_value.and_then(parse_port_override) {
        Some(port) => ServerConfig::new(flavour, port),
        None => ServerConfig::for_plugin_on_both_ports(flavour),
    }
}

/// [`server_config`] over the process environment, logging an override that is not a port.
pub fn server_config_from_environment(flavour: Flavour) -> ServerConfig {
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
    server_config(flavour, override_value.as_deref())
}

/// The port an override value names, or `None` for anything but a number from 1 to 65535.
fn parse_port_override(value: &str) -> Option<u16> {
    value.trim().parse::<u16>().ok().filter(|port| *port != 0)
}
