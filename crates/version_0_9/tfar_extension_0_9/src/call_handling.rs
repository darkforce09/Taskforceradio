//! How the 0.9 extension answers one `callExtension`: every call is a sync round trip, and each
//! way the round trip can fail has the reply string of the original named-pipe extension.
//!
//! **Role:** [`answer_call`] turns the call text into the reply Arma reads, over any
//! [`GameLinkCalls`] (the real [`GameLinkClient`] in the DLL, a fake in the unit tests); the
//! [`PIPE_NOT_OPENED`], [`PIPE_CLOSED_BY_TEAMSPEAK`] and [`NOT_CONNECTED_TO_TEAMSPEAK`] strings are
//! the ones the game link spec's "Game-facing strings" table gives for 0.9.
//!
//! **Position:** called by the `RVExtension` handler at the crate root; calls the client through
//! [`GameLinkCalls`].
//!
//! **Signals and state:** none of its own; the client keeps the session.
//!
//! **Invariants:** empty input sends nothing and replies empty; any other input is sent whole as
//! one `SYNC_REQ` (a trailing `~` has no meaning in 0.9 and is sent as is); a reply is the
//! plugin's text byte for byte where it is UTF-8.

use tfar_game_link::client::{GameLinkClient, SyncOutcome};

#[cfg(test)]
#[path = "tests/call_handling.rs"]
mod tests;

/// The reply when no session is up and none could be opened (the connect failed or the backoff
/// runs): the original's `ERROR_PIPE_LISTENING` reply.
pub const PIPE_NOT_OPENED: &str = "Pipe not opened from TS plugin";

/// The reply when the plugin closed the connection during the call: the original's
/// `ERROR_NO_DATA` reply.
pub const PIPE_CLOSED_BY_TEAMSPEAK: &str = "Pipe was closed by TS";

/// The reply when the plugin's answer did not arrive within the sync timeout: the original's
/// reply to Win32 error 230.
pub const NOT_CONNECTED_TO_TEAMSPEAK: &str = "Not connected to TeamSpeak";

/// The game link calls the 0.9 extension makes, so [`answer_call`] runs over the real client or a
/// test fake.
pub trait GameLinkCalls {
    /// Sends `text` as one `SYNC_REQ` and waits for its answer; see
    /// [`GameLinkClient::sync_request`].
    fn sync_request(&self, text: &[u8]) -> SyncOutcome;
}

impl GameLinkCalls for GameLinkClient {
    fn sync_request(&self, text: &[u8]) -> SyncOutcome {
        GameLinkClient::sync_request(self, text)
    }
}

/// The reply Arma reads for the call `input`.
///
/// | Situation | Reply |
/// |---|---|
/// | empty input | empty, nothing sent |
/// | the plugin answered | the plugin's text |
/// | no session and none could be opened | [`PIPE_NOT_OPENED`] |
/// | the plugin closed the connection during the call | [`PIPE_CLOSED_BY_TEAMSPEAK`] |
/// | no answer within the sync timeout | [`NOT_CONNECTED_TO_TEAMSPEAK`] |
pub fn answer_call(link: &impl GameLinkCalls, input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    match link.sync_request(input.as_bytes()) {
        SyncOutcome::Reply(text) => reply_text(text),
        SyncOutcome::NoSession => PIPE_NOT_OPENED.to_owned(),
        SyncOutcome::ClosedDuringCall => PIPE_CLOSED_BY_TEAMSPEAK.to_owned(),
        SyncOutcome::TimedOut => NOT_CONNECTED_TO_TEAMSPEAK.to_owned(),
    }
}

/// The plugin's reply bytes as text: unchanged when they are UTF-8, decoded lossily otherwise.
fn reply_text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes)
        .unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned())
}
