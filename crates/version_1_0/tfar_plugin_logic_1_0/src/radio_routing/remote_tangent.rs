//! `apply_remote_tangent`: a remote radio press or release, routed by this crate's rules.

use std::time::Instant;

use crate::antennas::AntennaManager;
use crate::client_state::prelude::{SessionState, TangentBroadcast, TangentEffects};
use crate::plugin_settings::PluginSettings;

use crate::radio_routing::reception::is_over_radio;
use crate::radio_routing::routing_context::RoutingContext;

#[cfg(test)]
#[path = "tests/remote_tangent.rs"]
mod tests;

/// Takes a tangent broadcast from another player at `now`
/// ([`SessionState::apply_remote_tangent`]) with [`is_over_radio`] as the routing, so the
/// sounds, the reception count and the heard list follow the reception rules.
pub fn apply_remote_tangent(
    session: &mut SessionState,
    settings: &PluginSettings,
    antennas: &AntennaManager,
    tangent: &TangentBroadcast,
    now: Instant,
) -> Option<TangentEffects> {
    session.apply_remote_tangent(tangent, now, |session, sender, ignore| {
        let context = RoutingContext {
            session,
            settings,
            antennas,
            now,
        };
        is_over_radio(&context, sender, ignore)
    })
}
