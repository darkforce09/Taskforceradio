//! The mute of players without current data: the dead and the not-yet-positioned.

use tfar_client_state_0_9::{ClientId, ServerRadioData};

use super::mute_tracker::{MuteDecision, MuteTracker};

#[cfg(test)]
#[path = "tests/dead_players.rs"]
mod tests;

/// For every client of the local player's channel but the local player that has no current data
/// (a dead player's frame is invalid), the requests that mute it while the local player is alive
/// in serious mode and unmute it otherwise (`setMuteForDeadPlayers`,
/// `reference/tfar_0_9_12/ts/src/plugin.cpp:1356`).
pub fn set_mute_for_dead_players(
    server: &ServerRadioData,
    tracker: &mut MuteTracker,
    channel_clients: &[ClientId],
    my_client_id: ClientId,
    is_serious_mode: bool,
) -> MuteDecision {
    let should_mute = server.is_alive && is_serious_mode;
    let mut decision = MuteDecision::default();
    for &client_id in channel_clients {
        if client_id != my_client_id && !server.has_fresh_data(client_id) {
            decision.merge(tracker.decide(client_id, should_mute));
        }
    }
    decision
}
