//! Unit tests for the 1.0 call handling: every situation maps to the original's reply string, and
//! the focus is sent before the call.

use std::cell::{Cell, RefCell};
use std::time::Duration;

use crate::focus_watch::FOCUS_READ_INTERVAL;

use super::*;

/// What the fake link was asked to do, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
enum LinkCall {
    Sync(Vec<u8>),
    Async(Vec<u8>),
    Focus(bool),
}

/// A link with a fixed sync outcome, async result and needcfg value that records every call.
struct FakeLink {
    sync_outcome: SyncOutcome,
    async_sent: bool,
    needcfg: Cell<bool>,
    calls: RefCell<Vec<LinkCall>>,
}

impl FakeLink {
    /// A link with a live session: sync calls get `reply`, async calls are sent, needcfg is false.
    fn live(reply: &[u8]) -> Self {
        Self {
            sync_outcome: SyncOutcome::Reply(reply.to_vec()),
            async_sent: true,
            needcfg: Cell::new(false),
            calls: RefCell::new(Vec::new()),
        }
    }

    /// A link with no session and none to open.
    fn without_session() -> Self {
        Self {
            sync_outcome: SyncOutcome::NoSession,
            async_sent: false,
            needcfg: Cell::new(true),
            calls: RefCell::new(Vec::new()),
        }
    }

    fn with_sync_outcome(mut self, outcome: SyncOutcome) -> Self {
        self.sync_outcome = outcome;
        self
    }

    fn calls(&self) -> Vec<LinkCall> {
        self.calls.borrow().clone()
    }
}

impl GameLinkCalls for FakeLink {
    fn sync_request(&self, text: &[u8]) -> SyncOutcome {
        self.calls.borrow_mut().push(LinkCall::Sync(text.to_vec()));
        self.sync_outcome.clone()
    }

    fn send_async(&self, text: &[u8]) -> bool {
        self.calls.borrow_mut().push(LinkCall::Async(text.to_vec()));
        self.async_sent
    }

    fn send_focus(&self, arma_focused: bool) {
        self.calls.borrow_mut().push(LinkCall::Focus(arma_focused));
    }

    fn needcfg(&self) -> bool {
        self.needcfg.get()
    }
}

/// Answers `input` with a fresh focus watch and Arma not in front.
fn answer(link: &FakeLink, input: &str) -> String {
    answer_call(
        link,
        &mut FocusWatch::new(),
        Instant::now(),
        || false,
        input,
    )
}

/// The calls other than `FOCUS`.
fn calls_without_focus(link: &FakeLink) -> Vec<LinkCall> {
    link.calls()
        .into_iter()
        .filter(|call| !matches!(call, LinkCall::Focus(_)))
        .collect()
}

#[test]
fn empty_input_replies_empty_and_sends_nothing() {
    let link = FakeLink::live(b"PONG");
    assert_eq!(answer(&link, ""), "");
    assert_eq!(link.calls(), Vec::new());
}

#[test]
fn a_sync_reply_is_returned_unchanged() {
    let link = FakeLink::live(b"PONG");
    assert_eq!(answer(&link, "TS_INFO\tPING"), "PONG");
    assert_eq!(
        calls_without_focus(&link),
        vec![LinkCall::Sync(b"TS_INFO\tPING".to_vec())]
    );
}

#[test]
fn a_sync_call_without_a_session_replies_not_connected_to_teamspeak() {
    let link = FakeLink::without_session();
    assert_eq!(answer(&link, "TS_INFO\tPING"), "Not connected to TeamSpeak");
}

#[test]
fn a_sync_call_whose_session_closed_replies_not_connected_to_teamspeak() {
    let link = FakeLink::live(b"").with_sync_outcome(SyncOutcome::ClosedDuringCall);
    assert_eq!(answer(&link, "TS_INFO\tPING"), "Not connected to TeamSpeak");
}

#[test]
fn a_sync_timeout_replies_empty() {
    let link = FakeLink::live(b"").with_sync_outcome(SyncOutcome::TimedOut);
    assert_eq!(answer(&link, "POS\tdata"), "");
}

#[test]
fn a_data_frame_with_needcfg_replies_needcfg() {
    let link = FakeLink::live(b"");
    link.needcfg.set(true);
    assert_eq!(answer(&link, "DFRAME~"), "NEEDCFG");
    assert_eq!(
        calls_without_focus(&link),
        vec![LinkCall::Async(b"DFRAME".to_vec())]
    );
}

#[test]
fn a_data_frame_without_needcfg_replies_ok() {
    let link = FakeLink::live(b"");
    assert_eq!(answer(&link, "DFRAME~"), "OK");
}

#[test]
fn mission_end_replies_empty() {
    let link = FakeLink::live(b"");
    link.needcfg.set(true);
    assert_eq!(answer(&link, "MISSIONEND~"), "");
    assert_eq!(
        calls_without_focus(&link),
        vec![LinkCall::Async(b"MISSIONEND".to_vec())]
    );
}

#[test]
fn any_other_async_call_replies_ok_and_is_sent_without_its_tilde() {
    let link = FakeLink::live(b"");
    link.needcfg.set(true);
    assert_eq!(answer(&link, "SETCFG\tpluginTimeout\t4~"), "OK");
    assert_eq!(answer(&link, "TANGENT\tPRESSED~"), "OK");
    assert_eq!(
        calls_without_focus(&link),
        vec![
            LinkCall::Async(b"SETCFG\tpluginTimeout\t4".to_vec()),
            LinkCall::Async(b"TANGENT\tPRESSED".to_vec()),
        ]
    );
}

#[test]
fn only_one_trailing_tilde_is_removed() {
    let link = FakeLink::live(b"");
    assert_eq!(answer(&link, "RELEASE_ALL_TANGENTS~~"), "OK");
    assert_eq!(
        calls_without_focus(&link),
        vec![LinkCall::Async(b"RELEASE_ALL_TANGENTS~".to_vec())]
    );
}

#[test]
fn the_command_name_is_matched_whole_up_to_the_first_tab() {
    let link = FakeLink::live(b"");
    link.needcfg.set(true);
    assert_eq!(answer(&link, "DFRAMEX~"), "OK");
    assert_eq!(answer(&link, "MISSIONENDED~"), "OK");
    assert_eq!(answer(&link, "DFRAME\textra~"), "NEEDCFG");
}

#[test]
fn an_async_call_without_a_session_replies_not_connected_to_teamspeak() {
    let link = FakeLink::without_session();
    assert_eq!(answer(&link, "DFRAME~"), "Not connected to TeamSpeak");
    assert_eq!(answer(&link, "MISSIONEND~"), "Not connected to TeamSpeak");
    assert_eq!(answer(&link, "SETCFG\ta\tb~"), "Not connected to TeamSpeak");
}

#[test]
fn the_focus_is_sent_before_the_first_call() {
    let link = FakeLink::live(b"PONG");
    let mut focus = FocusWatch::new();
    assert_eq!(
        answer_call(&link, &mut focus, Instant::now(), || true, "TS_INFO\tPING"),
        "PONG"
    );
    assert_eq!(
        link.calls(),
        vec![
            LinkCall::Focus(true),
            LinkCall::Sync(b"TS_INFO\tPING".to_vec())
        ]
    );
}

#[test]
fn the_focus_is_sent_again_only_when_it_changed() {
    let link = FakeLink::live(b"OK");
    let mut focus = FocusWatch::new();
    let start = Instant::now();
    let later = start + FOCUS_READ_INTERVAL;
    answer_call(&link, &mut focus, start, || true, "A");
    answer_call(&link, &mut focus, later, || true, "B");
    answer_call(
        &link,
        &mut focus,
        later + Duration::from_millis(100),
        || false,
        "C",
    );
    answer_call(
        &link,
        &mut focus,
        later + FOCUS_READ_INTERVAL,
        || false,
        "D",
    );
    let focus_calls: Vec<LinkCall> = link
        .calls()
        .into_iter()
        .filter(|call| matches!(call, LinkCall::Focus(_)))
        .collect();
    assert_eq!(
        focus_calls,
        vec![LinkCall::Focus(true), LinkCall::Focus(false)]
    );
}

#[test]
fn empty_input_reads_no_focus() {
    let link = FakeLink::live(b"");
    let reads = Cell::new(0);
    let reply = answer_call(
        &link,
        &mut FocusWatch::new(),
        Instant::now(),
        || {
            reads.set(reads.get() + 1);
            true
        },
        "",
    );
    assert_eq!(reply, "");
    assert_eq!(reads.get(), 0);
}
