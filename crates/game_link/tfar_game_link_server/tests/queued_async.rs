//! `GameLinkServer::queue_async`: plugin-queued async messages share the FIFO queue of the `ASYNC`
//! frames, in call order, under the same `MISSIONEND` generations.

// `cfg(test)` puts the helpers under the test lint relief of the root `clippy.toml`.
#[cfg(test)]
mod support;

use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use support::{Event, PROMPT, RawClient, RecordingHandler};
use tfar_game_link_protocol::{Flavour, Frame};
use tfar_game_link_server::{GameLinkHandler, GameLinkServer, ServerConfig};

/// The sync call prefix that makes [`SchedulingHandler`] queue the rest of the call as an async
/// message from inside its sync answer, as a sync `POS` schedules its async apply.
const SCHEDULE_PREFIX: &[u8] = b"SCHEDULE\t";

/// A handler that records like [`RecordingHandler`] and answers `SCHEDULE\t<text>` by queueing
/// `<text>` through the server it is attached to.
struct SchedulingHandler {
    recorder: Arc<RecordingHandler>,
    server: Mutex<Weak<GameLinkServer>>,
}

#[cfg(test)]
impl GameLinkHandler for SchedulingHandler {
    fn sync_request(&self, text: &[u8]) -> Vec<u8> {
        if let Some(scheduled) = text.strip_prefix(SCHEDULE_PREFIX) {
            let server = self.server.lock().unwrap().upgrade().unwrap();
            assert!(server.queue_async(scheduled));
            return b"SCHEDULED".to_vec();
        }
        self.recorder.sync_request(text)
    }

    fn async_message(&self, text: &[u8]) {
        self.recorder.async_message(text);
    }

    fn focus_changed(&self, focused: bool) {
        self.recorder.focus_changed(focused);
    }

    fn session_opened(&self) {
        self.recorder.session_opened();
    }

    fn session_closed(&self) {
        self.recorder.session_closed();
    }
}

/// A 1.0 server whose handler can queue from its sync answers, with the recorder and a client
/// that has passed the handshake.
#[cfg(test)]
fn start_scheduling_server() -> (Arc<GameLinkServer>, Arc<RecordingHandler>, RawClient) {
    let recorder = RecordingHandler::new();
    let handler = Arc::new(SchedulingHandler {
        recorder: Arc::clone(&recorder),
        server: Mutex::new(Weak::new()),
    });
    let server = Arc::new(
        GameLinkServer::start(ServerConfig::new(Flavour::V1_0, 0), handler.clone()).unwrap(),
    );
    *handler.server.lock().unwrap() = Arc::downgrade(&server);
    let address = server.local_addr().unwrap();
    let (client, _) = RawClient::handshake(address, Flavour::V1_0);
    (server, recorder, client)
}

/// Shuts down a server the test holds the only strong reference to.
#[cfg(test)]
fn shut_down(server: Arc<GameLinkServer>) {
    Arc::into_inner(server)
        .expect("the handler holds the server weakly")
        .shutdown();
}

/// Waits until `count` async messages were delivered.
fn wait_for_async_count(recorder: &RecordingHandler, count: usize) -> bool {
    recorder.wait_until(PROMPT, |events| {
        events
            .iter()
            .filter(|event| matches!(event, Event::Async(_)))
            .count()
            == count
    })
}

#[test]
fn a_message_queued_from_a_sync_answer_runs_between_the_async_frames_around_it() {
    let (server, recorder, mut client) = start_scheduling_server();
    recorder.set_async_gate_closed(true);
    client.send_async(b"FIRST");
    assert_eq!(
        client.sync_call(1, b"SCHEDULE\tPOS_APPLY"),
        Frame::SyncResponse {
            id: 1,
            text: b"SCHEDULED".to_vec()
        }
    );
    client.send_async(b"THIRD");
    // Frames are handled in order, so after this answer all three messages are queued.
    assert!(matches!(
        client.sync_call(2, b"TS_INFO\tPING"),
        Frame::SyncResponse { id: 2, .. }
    ));
    recorder.set_async_gate_closed(false);
    assert!(wait_for_async_count(&recorder, 3));
    assert_eq!(
        recorder.async_texts(),
        [b"FIRST".to_vec(), b"POS_APPLY".to_vec(), b"THIRD".to_vec()]
    );
    shut_down(server);
}

#[test]
fn a_thousand_interleaved_frames_and_queued_messages_keep_call_order() {
    let (server, recorder, mut client) = start_scheduling_server();
    let mut expected = Vec::new();
    for index in 0..500_u32 {
        let framed = format!("FRAME\t{index}").into_bytes();
        client.send_async(&framed);
        expected.push(framed);
        let queued = format!("QUEUED\t{index}");
        let reply = client.sync_call(index, format!("SCHEDULE\t{queued}").as_bytes());
        assert!(matches!(reply, Frame::SyncResponse { .. }));
        expected.push(queued.into_bytes());
    }
    assert!(recorder.wait_until(Duration::from_secs(5), |events| {
        events
            .iter()
            .filter(|event| matches!(event, Event::Async(_)))
            .count()
            == 1000
    }));
    assert_eq!(recorder.async_texts(), expected);
    shut_down(server);
}

#[test]
fn a_later_missionend_frame_drops_a_queued_message_still_waiting() {
    let (server, recorder, mut client) = start_scheduling_server();
    recorder.set_async_gate_closed(true);
    client.send_async(b"RUNNING");
    assert!(recorder.wait_until(PROMPT, |events| {
        events.contains(&Event::Async(b"RUNNING".to_vec()))
    }));
    client.sync_call(1, b"SCHEDULE\tSTALE_APPLY");
    client.send_async(b"MISSIONEND");
    client.sync_call(2, b"TS_INFO\tPING");
    recorder.set_async_gate_closed(false);
    assert!(wait_for_async_count(&recorder, 2));
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(
        recorder.async_texts(),
        [b"RUNNING".to_vec(), b"MISSIONEND".to_vec()]
    );
    shut_down(server);
}

#[test]
fn a_queued_missionend_ends_the_mission_like_the_frame() {
    let (server, recorder, mut client) = start_scheduling_server();
    client.send_async(b"DFRAME");
    assert!(wait_for_async_count(&recorder, 1));
    assert!(server.last_game_message().is_some());
    recorder.set_async_gate_closed(true);
    client.send_async(b"HELD");
    assert!(wait_for_async_count(&recorder, 2));
    client.send_async(b"STALE");
    client.sync_call(1, b"TS_INFO\tPING");
    let queued = server.queue_async(b"MISSIONEND");
    let last_game_message = server.last_game_message();
    // Open the gate before any assertion, so a failure ends the test instead of holding the
    // worker inside the handler.
    recorder.set_async_gate_closed(false);
    assert!(queued);
    assert_eq!(last_game_message, None, "MISSIONEND resets it");
    assert!(wait_for_async_count(&recorder, 3));
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(
        recorder.async_texts(),
        [b"DFRAME".to_vec(), b"HELD".to_vec(), b"MISSIONEND".to_vec()]
    );
    shut_down(server);
}

#[test]
fn a_message_queued_without_a_session_still_runs_and_counts_as_a_game_message() {
    let recorder = RecordingHandler::new();
    let server =
        GameLinkServer::start(ServerConfig::new(Flavour::V1_0, 0), recorder.clone()).unwrap();
    assert!(!server.is_session_up());
    assert!(server.queue_async(b"POS_APPLY"));
    assert!(wait_for_async_count(&recorder, 1));
    assert_eq!(recorder.async_texts(), [b"POS_APPLY".to_vec()]);
    assert!(server.last_game_message().is_some());
    server.shutdown();
}
