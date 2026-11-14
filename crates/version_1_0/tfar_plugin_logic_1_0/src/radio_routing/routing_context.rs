//! `RoutingContext`: what every reception rule reads.

use std::time::Instant;

use crate::antennas::AntennaManager;
use crate::client_state::{GameData, SessionState};
use crate::plugin_settings::PluginSettings;

/// The state a reception rule reads: the session's clients, the local player's game data, the
/// settings, the radio towers and the time. Built under the game lock and the session lock and
/// dropped before either is released.
#[derive(Clone, Copy, Debug)]
pub struct RoutingContext<'a> {
    /// The clients; the local player is `session.clients.own()`.
    pub session: &'a SessionState,
    /// What the game reports about the local player: radios, liveness, data frame, speakers.
    pub game: &'a GameData,
    /// The plugin settings (`intercomEnabled`, `full_duplex`, `disableAutomaticMute`).
    pub settings: &'a PluginSettings,
    /// The radio towers.
    pub antennas: &'a AntennaManager,
    /// The time positions are extrapolated to and liveness is judged at.
    pub now: Instant,
}
