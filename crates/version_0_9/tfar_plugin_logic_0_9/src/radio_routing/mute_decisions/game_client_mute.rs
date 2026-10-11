//! The serious-mode mute of one player: whether the local player may hear them in TeamSpeak.
//!
//! **Role:** `setGameClientMuteStatus` (`reference/tfar_0_9_12/ts/src/plugin.cpp:1188-1218`) as a
//! decision: in serious mode a player is muted unless both have current data, the local player
//! is alive, and the player is heard over a radio or talks within voice range.
//!
//! **Position:** called by the plugin after every `POS` of another player, every incoming
//! `VOLUME` and each reception of an incoming push-to-talk; the result goes through the
//! `MuteTracker`.
//!
//! **Signals and state:** `game_client_mute` reads the server state; `set_game_client_mute_status`
//! also resets the voice state of a player it mutes (`reset_voices`: filter memory cleared,
//! nothing freed, since this runs on every `POS` of a silent player) and updates the tracker.
//!
//! **Invariants:** outside serious mode the wanted state is always unmuted; a player without an
//! entry is muted and no voice state is touched; the requests follow the tracker's rules.

use crate::client_state::{ClientId, ServerRadioData};

use super::mute_tracker::{MuteDecision, MuteTracker};
use crate::radio_routing::distances::raw_distance;
use crate::radio_routing::heard_via::IgnoreTangents;
use crate::radio_routing::reception::is_over_radio;

#[cfg(test)]
#[path = "tests/game_client_mute.rs"]
mod tests;

/// What the plugin knows from TeamSpeak about the player whose mute it decides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MuteQuery {
    /// The player's client id.
    pub client_id: ClientId,
    /// The local player's client id.
    pub my_client_id: ClientId,
    /// Whether the player sits in the serious mode channel (see [`crate::radio_routing::is_serious_mode`]).
    pub is_in_serious_channel: bool,
    /// TeamSpeak's talking flag for the player (the local player's own flag when the player is
    /// the local player).
    pub is_talking_in_teamspeak: bool,
}

/// The wanted mute state of one player.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameClientMute {
    /// Whether the player is to be muted.
    pub should_mute: bool,
    /// The nickname whose voice state a mute resets: the player's current entry, or `None` when
    /// the player has none.
    pub resets_voices_of: Option<String>,
}

/// The wanted mute state of `query`'s player.
pub fn game_client_mute(server: &ServerRadioData, query: &MuteQuery) -> GameClientMute {
    if !query.is_in_serious_channel {
        return GameClientMute {
            should_mute: false,
            resets_voices_of: None,
        };
    }
    let player = server.fresh_client_by_id(query.client_id);
    let listener = server.fresh_client_by_id(query.my_client_id);
    let should_mute = match (player, listener) {
        (Some((nickname, player)), Some((_, listener))) if server.is_alive => {
            let is_heard_over_radio =
                !is_over_radio(server, nickname, player, listener, IgnoreTangents::NONE).is_empty();
            let is_talking = player.is_talking_now || query.is_talking_in_teamspeak;
            !is_heard_over_radio
                && (raw_distance(server, player) > f64::from(player.voice_volume) || !is_talking)
        }
        _ => true,
    };
    GameClientMute {
        should_mute,
        resets_voices_of: player
            .filter(|_| should_mute)
            .map(|(nickname, _)| nickname.to_owned()),
    }
}

/// Decides `query`'s player's mute state, resets their voice state when muting them, and returns
/// the requests `tracker` needs to reach it.
pub fn set_game_client_mute_status(
    server: &mut ServerRadioData,
    tracker: &mut MuteTracker,
    query: &MuteQuery,
) -> MuteDecision {
    let wanted = game_client_mute(server, query);
    if let Some(nickname) = &wanted.resets_voices_of
        && let Some(player) = server.client_mut(nickname)
    {
        player.reset_voices();
    }
    tracker.decide(query.client_id, wanted.should_mute)
}
