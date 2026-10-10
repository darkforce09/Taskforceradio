//! Unit tests for the 0.9 call handling: every sync outcome maps to the original's reply string.

use std::cell::RefCell;

use super::*;

/// A link that answers every sync request with one fixed outcome and records what was sent.
struct FakeLink {
    outcome: SyncOutcome,
    sent: RefCell<Vec<Vec<u8>>>,
}

impl FakeLink {
    fn answering(outcome: SyncOutcome) -> Self {
        Self {
            outcome,
            sent: RefCell::new(Vec::new()),
        }
    }

    fn sent(&self) -> Vec<Vec<u8>> {
        self.sent.borrow().clone()
    }
}

impl GameLinkCalls for FakeLink {
    fn sync_request(&self, text: &[u8]) -> SyncOutcome {
        self.sent.borrow_mut().push(text.to_vec());
        self.outcome.clone()
    }
}

#[test]
fn empty_input_replies_empty_and_sends_nothing() {
    let link = FakeLink::answering(SyncOutcome::Reply(b"PONG".to_vec()));
    assert_eq!(answer_call(&link, ""), "");
    assert_eq!(link.sent(), Vec::<Vec<u8>>::new());
}

#[test]
fn a_plugin_reply_is_returned_unchanged() {
    let link = FakeLink::answering(SyncOutcome::Reply(b"PONG".to_vec()));
    assert_eq!(answer_call(&link, "TS_INFO\tPING"), "PONG");
    assert_eq!(link.sent(), vec![b"TS_INFO\tPING".to_vec()]);
}

#[test]
fn an_empty_plugin_reply_stays_empty() {
    let link = FakeLink::answering(SyncOutcome::Reply(Vec::new()));
    assert_eq!(answer_call(&link, "POS\tx"), "");
}

#[test]
fn a_reply_that_is_not_utf8_is_decoded_lossily() {
    let link = FakeLink::answering(SyncOutcome::Reply(vec![b'O', 0xFF, b'K']));
    assert_eq!(answer_call(&link, "TS_INFO\tPING"), "O\u{fffd}K");
}

#[test]
fn no_session_replies_pipe_not_opened() {
    let link = FakeLink::answering(SyncOutcome::NoSession);
    assert_eq!(
        answer_call(&link, "TS_INFO\tPING"),
        "Pipe not opened from TS plugin"
    );
}

#[test]
fn a_close_during_the_call_replies_pipe_closed_by_ts() {
    let link = FakeLink::answering(SyncOutcome::ClosedDuringCall);
    assert_eq!(answer_call(&link, "TS_INFO\tPING"), "Pipe was closed by TS");
}

#[test]
fn a_sync_timeout_replies_not_connected_to_teamspeak() {
    let link = FakeLink::answering(SyncOutcome::TimedOut);
    assert_eq!(
        answer_call(&link, "TS_INFO\tPING"),
        "Not connected to TeamSpeak"
    );
}

#[test]
fn a_trailing_tilde_is_sent_as_is_in_a_sync_request() {
    let link = FakeLink::answering(SyncOutcome::Reply(b"OK".to_vec()));
    assert_eq!(answer_call(&link, "DFRAME~"), "OK");
    assert_eq!(link.sent(), vec![b"DFRAME~".to_vec()]);
}
