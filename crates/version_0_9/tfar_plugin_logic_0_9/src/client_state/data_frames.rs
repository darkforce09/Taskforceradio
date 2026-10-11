//! Data frames and expiry: which players' data is current, and when an entry goes.
//!
//! **Role:** answer whether a client has current data (`hasClientData`,
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:584`), find that client's current entry, and remove
//! the entries that have gone stale (`removeExpiredPositions`, plugin.cpp:1938).
//!
//! **Position:** methods of `ServerRadioData`; the routing, the mute decisions and the voice
//! pipeline gate on `has_fresh_data`; the plugin's service tick calls `remove_expired` every 4 s.
//!
//! **Signals and state:** reads the data frames and position times; `remove_expired` removes
//! entries and drops their sound state, on the service thread under the server lock.
//!
//! **Invariants:** a frame is current within one of the current data frame, the invalid frame
//! never; a lookup by client id returns a current entry whenever `has_fresh_data` is true for
//! that id, and the entry most recently positioned when several current entries share the id;
//! time comes in as `now`, never from the clock.

use std::time::{Duration, Instant};

use crate::client_state::client_data::ClientData;
use crate::client_state::client_id::ClientId;
use crate::client_state::server_radio_data::{INVALID_DATA_FRAME, ServerRadioData};

#[cfg(test)]
#[path = "tests/data_frames.rs"]
mod tests;

/// How long an entry lives after the last `POS` or push-to-talk of its player
/// (`MILLIS_TO_EXPIRE * 5`, plugin.cpp:1948).
pub const POSITION_EXPIRY: Duration = Duration::from_secs(20);

/// The widest distance between a player's data frame and the current one that still counts as
/// current.
pub const DATA_FRAME_WINDOW: u32 = 1;

impl ServerRadioData {
    /// Whether `data_frame` lies within [`DATA_FRAME_WINDOW`] of the current data frame. The
    /// invalid frame, of an entry without a `POS` or of a killed player, is never current, also
    /// outside a game, where the current frame sits at or just above it.
    pub fn is_frame_current(&self, data_frame: i32) -> bool {
        data_frame != INVALID_DATA_FRAME
            && self.current_data_frame.abs_diff(data_frame) <= DATA_FRAME_WINDOW
    }

    /// Whether some entry names `client_id` and carries a current data frame.
    pub fn has_fresh_data(&self, client_id: ClientId) -> bool {
        self.fresh_client_by_id(client_id).is_some()
    }

    /// The nickname and entry of `client_id` with a current data frame, or `None`.
    ///
    /// After a rename the old nickname's entry keeps the client id until it expires; the
    /// original's `getClientData` then returns the first entry by nickname order, stale or not,
    /// while `hasClientData` passed on the other. This returns only a current entry, and of
    /// several the one positioned last.
    pub fn fresh_client_by_id(&self, client_id: ClientId) -> Option<(&str, &ClientData)> {
        self.clients
            .iter()
            .filter(|(_, client)| {
                client.client_id == client_id && self.is_frame_current(client.data_frame)
            })
            .fold(
                None,
                |chosen: Option<(&String, &ClientData)>, candidate| match chosen {
                    Some(best) if best.1.position_time >= candidate.1.position_time => Some(best),
                    _ => Some(candidate),
                },
            )
            .map(|(nickname, client)| (nickname.as_str(), client))
    }

    /// The nickname of the entry [`ServerRadioData::fresh_client_by_id`] returns.
    pub fn fresh_nickname_of(&self, client_id: ClientId) -> Option<&str> {
        self.fresh_client_by_id(client_id)
            .map(|(nickname, _)| nickname)
    }

    /// The entry [`ServerRadioData::fresh_client_by_id`] returns, to change.
    pub fn fresh_client_by_id_mut(&mut self, client_id: ClientId) -> Option<&mut ClientData> {
        let nickname = self.fresh_nickname_of(client_id)?.to_owned();
        self.clients.get_mut(&nickname)
    }

    /// Removes every entry whose player sent no `POS` or push-to-talk within
    /// [`POSITION_EXPIRY`] before `now` (or never), or whose data frame is more than
    /// [`DATA_FRAME_WINDOW`] from the current one; returns the removed entries by nickname, in
    /// order. The caller drops them after releasing the server lock, so freeing their sound state
    /// (direction sources of about two megabytes each) never holds up the audio thread.
    pub fn remove_expired(&mut self, now: Instant) -> Vec<(String, ClientData)> {
        let current_data_frame = self.current_data_frame;
        self.clients
            .extract_if(.., |_, client| {
                let is_timed_out = client.position_time.is_none_or(|positioned| {
                    now.saturating_duration_since(positioned) > POSITION_EXPIRY
                });
                let is_frame_stale =
                    current_data_frame.abs_diff(client.data_frame) > DATA_FRAME_WINDOW;
                is_timed_out || is_frame_stale
            })
            .collect()
    }
}
