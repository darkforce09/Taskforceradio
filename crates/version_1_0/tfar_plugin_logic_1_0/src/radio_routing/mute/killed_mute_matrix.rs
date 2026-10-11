//! `killed_mute_matrix`: who hears whom after a kill.

use crate::client_state::prelude::TeamspeakClientId;

use crate::radio_routing::routing_context::RoutingContext;

#[cfg(test)]
#[path = "tests/killed_mute_matrix.rs"]
mod tests;

/// Clients to mute and to unmute.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MuteChanges {
    /// The clients to mute.
    pub mute: Vec<TeamspeakClientId>,
    /// The clients to unmute.
    pub unmute: Vec<TeamspeakClientId>,
}

/// The mutes after a `KILLED` message over `channel_clients`, the clients of the local
/// player's channel (`processUnitKilled`, `reference/tfar_1_0/ts/src/CommandProcessor.cpp:
/// 552-587`). The local player is never in the result.
///
/// - Outside serious mode a dead local player hears everyone: every client is unmuted.
/// - Otherwise, unless `disableAutomaticMute`: in serious mode the living hear only the living
///   and the dead only the dead; outside it (the local player alive) everyone is unmuted. An
///   unknown client counts as dead.
pub fn killed_mute_matrix(
    context: &RoutingContext<'_>,
    is_serious_mode: bool,
    channel_clients: &[TeamspeakClientId],
) -> MuteChanges {
    let session = context.session;
    let own = session.clients.own_client_id();
    let others = channel_clients
        .iter()
        .copied()
        .filter(|client| Some(*client) != own);
    let is_player_alive = session.game.is_alive;
    if !is_serious_mode && !is_player_alive {
        return MuteChanges {
            mute: Vec::new(),
            unmute: others.collect(),
        };
    }
    if context.settings.disable_automatic_mute() {
        return MuteChanges::default();
    }
    let mut changes = MuteChanges::default();
    for client in others {
        let is_client_alive = session.is_client_alive(client, context.now);
        let mute = is_serious_mode && is_client_alive != is_player_alive;
        if mute {
            changes.mute.push(client);
        } else {
            changes.unmute.push(client);
        }
    }
    changes
}
