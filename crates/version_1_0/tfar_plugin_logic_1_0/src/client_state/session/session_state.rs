//! `SessionState`: the clients of one server connection, behind that connection's lock.
//!
//! **Role:** hold the client directory of one server connection and answer the queries and
//! sweeps over it; what needs the game's data frame takes it as an argument.
//!
//! **Position:** a module of the crate's `session`; owned by the plugin app, one per server
//! connection; read by the routing crate and the voice pipeline.
//!
//! **Signals and state:** the session itself, behind the app's `parking_lot` mutex per server;
//! the game link worker, the TeamSpeak event thread, the service thread and the audio thread take
//! it. The `GameData` it is applied with lives once per plugin, behind its own lock.
//!
//! **Invariants:** queries are pure; only [`SessionState::sweep_expired`] expires clients.

use std::time::Instant;

use crate::game_commands::replies::{UNKNOWN_CLIENT_SPEAKING_CODE, speaking_code};
use crate::plugin_commands::TeamspeakClientId;

use crate::client_state::client_directory::ClientDirectory;
use crate::client_state::data_frame::DataFrame;

#[cfg(test)]
#[path = "tests/session_state.rs"]
mod tests;

/// The clients of one server connection.
///
/// The plugin app keeps one behind one `parking_lot` mutex per server connection, and one
/// [`crate::client_state::GameData`] for the whole plugin, as the original keeps one `gameData`
/// (`reference/tfar_1_0/ts/src/task_force_radio.hpp:143`). Every apply function is pure state
/// work, with the game data and time passed in, and returns what the app must do in TeamSpeak
/// after releasing the locks; no reference into the state outlives the guard.
#[derive(Debug, Default)]
pub struct SessionState {
    /// The clients of the server.
    pub clients: ClientDirectory,
}

impl SessionState {
    /// An empty session: no client.
    pub fn new() -> Self {
        Self::default()
    }

    /// The two-character speaking code of the client `nickname` for the `POS` and
    /// `IS_SPEAKING` replies: talking now or holding a tangent, and receiving a transmission;
    /// `00` for an unknown client (`CommandProcessor.cpp:89-101`).
    pub fn speaking_code(&self, nickname: &str) -> [u8; 2] {
        self.clients
            .get(nickname)
            .map_or(UNKNOWN_CLIENT_SPEAKING_CODE, |client| {
                speaking_code(
                    client.is_talking_now || client.is_transmitting(),
                    client.is_receiving_transmission(),
                )
            })
    }

    /// Whether the client `client_id` is alive at `now` in the game's `current_frame`, by the
    /// pure rule of [`crate::client_state::ClientData::is_alive`]; an unknown client is not.
    pub fn is_client_alive(
        &self,
        client_id: TeamspeakClientId,
        current_frame: DataFrame,
        now: Instant,
    ) -> bool {
        self.clients
            .by_id(client_id)
            .is_some_and(|client| client.is_alive(now, current_frame))
    }

    /// Expires every client that is no longer alive at `now` in the game's `current_frame`: its
    /// data frame becomes invalid, so it stays dead until its next position. The service thread
    /// calls this every 4 s; returns the expired clients, whose mute state the app recomputes.
    pub fn sweep_expired(
        &mut self,
        current_frame: DataFrame,
        now: Instant,
    ) -> Vec<TeamspeakClientId> {
        let mut expired = Vec::new();
        for client in self.clients.iter_mut() {
            if client.data_frame.is_valid() && !client.is_alive(now, current_frame) {
                client.data_frame = DataFrame::INVALID;
                expired.push(client.client_id());
            }
        }
        expired
    }

    /// The client `client_id` runs TFAR and plays, at `now`: the cached answer, or the answer
    /// read from `metadata` ([`crate::client_state::ClientData::plugin_enabled_for`]); an unknown client is
    /// not.
    pub fn plugin_enabled_for(
        &mut self,
        client_id: TeamspeakClientId,
        metadata: Option<&str>,
        now: Instant,
    ) -> bool {
        self.clients
            .by_id_mut(client_id)
            .is_some_and(|client| client.plugin_enabled_for(metadata, now))
    }

    /// Whether the client `client_id`'s metadata must be read to answer
    /// [`SessionState::plugin_enabled_for`] at `now`; the app reads it with the lock released.
    pub fn needs_plugin_metadata(&self, client_id: TeamspeakClientId, now: Instant) -> bool {
        self.clients
            .by_id(client_id)
            .is_some_and(|client| client.cached_plugin_enabled(now).is_none())
    }
}
