//! `RoutingContext`: what every reception rule reads.

use std::time::Instant;

use tfar_antennas_1_0::AntennaManager;
use tfar_client_state_1_0::SessionState;
use tfar_plugin_settings_1_0::PluginSettings;

/// The state a reception rule reads: the session (game data and clients), the settings, the
/// radio towers and the time. Built under the session lock and dropped before it is released.
#[derive(Clone, Copy, Debug)]
pub struct RoutingContext<'a> {
    /// The game data and the clients; the local player is `session.clients.own()`.
    pub session: &'a SessionState,
    /// The plugin settings (`intercomEnabled`, `full_duplex`, `disableAutomaticMute`).
    pub settings: &'a PluginSettings,
    /// The radio towers.
    pub antennas: &'a AntennaManager,
    /// The time positions are extrapolated to and liveness is judged at.
    pub now: Instant,
}
