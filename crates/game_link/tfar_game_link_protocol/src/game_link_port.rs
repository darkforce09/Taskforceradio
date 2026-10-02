//! The loopback port each plugin listens on.

use crate::flavour::Flavour;

#[cfg(test)]
#[path = "tests/game_link_port.rs"]
mod tests;

/// The 0.9 plugin's port.
pub const PORT_0_9: u16 = 47810;
/// The 0.9 plugin's port for a `-tfdebug` Arma instance.
pub const PORT_0_9_DEBUG: u16 = 47811;
/// The 1.0 plugin's port.
pub const PORT_1_0: u16 = 47820;
/// The 1.0 plugin's port for a `-tfdebug` Arma instance.
pub const PORT_1_0_DEBUG: u16 = 47821;

/// The `127.0.0.1` port of the plugin for `flavour`; `debug` picks the port of a `-tfdebug` instance.
pub const fn port(flavour: Flavour, debug: bool) -> u16 {
    match (flavour, debug) {
        (Flavour::V0_9, false) => PORT_0_9,
        (Flavour::V0_9, true) => PORT_0_9_DEBUG,
        (Flavour::V1_0, false) => PORT_1_0,
        (Flavour::V1_0, true) => PORT_1_0_DEBUG,
    }
}
