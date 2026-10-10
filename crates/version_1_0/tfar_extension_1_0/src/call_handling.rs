//! How the 1.0 extension answers one `callExtension`: sync calls wait for the plugin, `~` calls go
//! async and are answered at once, and the focus state rides along.
//!
//! **Role:** [`answer_call`] turns the call text into the reply Arma reads, over any
//! [`GameLinkCalls`] (the real [`GameLinkClient`] in the DLL, a fake in the unit tests). The reply
//! strings are the game link spec's 1.0 column of the "Game-facing strings" table:
//! [`NOT_CONNECTED_TO_TEAMSPEAK`] without a live session, [`ASYNC_ACCEPTED`] and
//! [`CONFIGURATION_NEEDED`] for async calls, empty for `MISSIONEND~` and a sync timeout.
//!
//! **Position:** called by the `RVExtension` handler at the crate root; calls the client through
//! [`GameLinkCalls`] and reads the focus through [`FocusWatch`].
//!
//! **Signals and state:** none of its own; the client keeps the session and the needcfg value,
//! the caller keeps the [`FocusWatch`].
//!
//! **Invariants:** empty input sends nothing, reads no focus and replies empty; an async call is
//! sent with exactly one trailing `~` removed and is answered without waiting for the plugin; the
//! focus is checked before the call is sent, so a new session carries `FOCUS` before the call.

use std::time::Instant;

use tfar_game_link_client::{GameLinkClient, SyncOutcome};

use crate::focus_watch::FocusWatch;

#[cfg(test)]
#[path = "tests/call_handling.rs"]
mod tests;

/// The reply to every call, sync or async, while no live session is up and none could be opened
/// (the original's reply when the plugin's tick is older than 1000 ms).
pub const NOT_CONNECTED_TO_TEAMSPEAK: &str = "Not connected to TeamSpeak";

/// The reply to an async call the plugin was sent, except `DFRAME` with needcfg and
/// `MISSIONEND`.
pub const ASYNC_ACCEPTED: &str = "OK";

/// The reply to `DFRAME~` while the plugin's latest `STATUS` says it needs the settings.
pub const CONFIGURATION_NEEDED: &str = "NEEDCFG";

/// The suffix that marks an async call.
pub const ASYNC_MARKER: char = '~';

/// The async command that asks whether the plugin needs the settings.
pub const DATA_FRAME_COMMAND: &str = "DFRAME";

/// The async command that ends the mission; its reply is empty.
pub const MISSION_END_COMMAND: &str = "MISSIONEND";

/// The game link calls the 1.0 extension makes, so [`answer_call`] runs over the real client or a
/// test fake.
pub trait GameLinkCalls {
    /// Sends `text` as one `SYNC_REQ` and waits for its answer; see
    /// [`GameLinkClient::sync_request`].
    fn sync_request(&self, text: &[u8]) -> SyncOutcome;

    /// Sends `text` as one `ASYNC` frame; true when it was sent.
    fn send_async(&self, text: &[u8]) -> bool;

    /// Hands the focus state to the client, which sends it now or after the next handshake.
    fn send_focus(&self, arma_focused: bool);

    /// Whether the plugin's latest `STATUS` asked for the settings.
    fn needcfg(&self) -> bool;
}

impl GameLinkCalls for GameLinkClient {
    fn sync_request(&self, text: &[u8]) -> SyncOutcome {
        GameLinkClient::sync_request(self, text)
    }

    fn send_async(&self, text: &[u8]) -> bool {
        match GameLinkClient::send_async(self, text) {
            Ok(()) => true,
            Err(error) => {
                tracing::debug!(%error, "an async call did not reach the game link plugin");
                false
            }
        }
    }

    fn send_focus(&self, arma_focused: bool) {
        // The client keeps the value and sends it after the next handshake when this send fails.
        if let Err(error) = GameLinkClient::send_focus(self, arma_focused) {
            tracing::debug!(%error, "the focus state did not reach the game link plugin");
        }
    }

    fn needcfg(&self) -> bool {
        GameLinkClient::needcfg(self)
    }
}

/// The reply Arma reads for the call `input`, made at `now`.
///
/// Before a non-empty call is sent, `focus` decides at `now` whether to read the foreground window
/// with `read_focus` and whether to send the value.
///
/// | Situation | Reply |
/// |---|---|
/// | empty input | empty, nothing sent |
/// | `DFRAME~` sent, the latest `STATUS` says needcfg | [`CONFIGURATION_NEEDED`] |
/// | `MISSIONEND~` sent | empty |
/// | any other `~` call sent | [`ASYNC_ACCEPTED`] |
/// | a `~` call with no session up and none opened | [`NOT_CONNECTED_TO_TEAMSPEAK`] |
/// | a sync call the plugin answered | the plugin's text |
/// | a sync call with no session, or the session closed during it | [`NOT_CONNECTED_TO_TEAMSPEAK`] |
/// | a sync call with no answer within the sync timeout | empty |
pub fn answer_call(
    link: &impl GameLinkCalls,
    focus: &mut FocusWatch,
    now: Instant,
    read_focus: impl FnOnce() -> bool,
    input: &str,
) -> String {
    if input.is_empty() {
        return String::new();
    }
    if let Some(arma_focused) = focus.poll(now, read_focus) {
        link.send_focus(arma_focused);
    }
    match input.strip_suffix(ASYNC_MARKER) {
        Some(message) => answer_async_call(link, message),
        None => answer_sync_call(link, input),
    }
}

/// Sends `message` (the call without its `~`) as `ASYNC` and answers at once.
fn answer_async_call(link: &impl GameLinkCalls, message: &str) -> String {
    if !link.send_async(message.as_bytes()) {
        return NOT_CONNECTED_TO_TEAMSPEAK.to_owned();
    }
    match command_name(message) {
        DATA_FRAME_COMMAND if link.needcfg() => CONFIGURATION_NEEDED.to_owned(),
        MISSION_END_COMMAND => String::new(),
        _ => ASYNC_ACCEPTED.to_owned(),
    }
}

/// Sends `input` as `SYNC_REQ` and answers with the plugin's text, or the string of the failure.
fn answer_sync_call(link: &impl GameLinkCalls, input: &str) -> String {
    match link.sync_request(input.as_bytes()) {
        SyncOutcome::Reply(text) => String::from_utf8(text)
            .unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned()),
        SyncOutcome::NoSession | SyncOutcome::ClosedDuringCall => {
            NOT_CONNECTED_TO_TEAMSPEAK.to_owned()
        }
        SyncOutcome::TimedOut => String::new(),
    }
}

/// The command name of a call: its text up to the first TAB.
fn command_name(message: &str) -> &str {
    message.split('\t').next().unwrap_or(message)
}
