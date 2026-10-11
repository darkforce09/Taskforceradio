//! Sync answers, the async FIFO, `MISSIONEND`, `FOCUS` and the last game message time of
//! `GameLinkServer`, over real loopback sockets.

// `cfg(test)` puts the helpers under the test lint relief of the root `clippy.toml`.
#[cfg(test)]
mod support;

use std::thread;
use std::time::Duration;

use support::{Event, PROMPT, RawClient, start_server};
use tfar_game_link::protocol::{Flavour, Frame};

#[test]
fn a_sync_request_is_answered_with_the_handler_reply_and_the_same_id() {
    let (server, handler, address) = start_server(Flavour::V1_0);
    let (mut client, _) = RawClient::handshake(address, Flavour::V1_0);
    assert_eq!(
        client.sync_call(1, b"TS_INFO\tPING"),
        Frame::SyncResponse {
            id: 1,
            text: b"PONG".to_vec()
        }
    );
    assert_eq!(
        client.sync_call(2, &[0xFF, b'\t', 0x00]),
        Frame::SyncResponse {
            id: 2,
            text: [b"ECHO ".as_slice(), &[0xFF, b'\t', 0x00]].concat()
        },
        "text bytes pass through untouched"
    );
    assert!(
        handler
            .events()
            .contains(&Event::SyncRequest(b"TS_INFO\tPING".to_vec()))
    );
    server.shutdown();
}

#[test]
fn the_0_9_flavour_answers_sync_requests_too() {
    let (server, _handler, address) = start_server(Flavour::V0_9);
    let (mut client, needcfg) = RawClient::handshake(address, Flavour::V0_9);
    assert!(!needcfg);
    assert_eq!(
        client.sync_call(7, b"TS_INFO\tPING"),
        Frame::SyncResponse {
            id: 7,
            text: b"PONG".to_vec()
        }
    );
    server.shutdown();
}

#[test]
fn a_thousand_async_messages_are_delivered_in_arrival_order() {
    let (server, handler, address) = start_server(Flavour::V1_0);
    let (mut client, _) = RawClient::handshake(address, Flavour::V1_0);
    let sent: Vec<Vec<u8>> = (0..1000)
        .map(|index| format!("POS\t{index}").into_bytes())
        .collect();
    for text in &sent {
        client.send_async(text);
    }
    assert!(handler.wait_until(Duration::from_secs(5), |events| {
        events
            .iter()
            .filter(|event| matches!(event, Event::Async(_)))
            .count()
            == 1000
    }));
    assert_eq!(handler.async_texts(), sent);
    server.shutdown();
}

#[test]
fn missionend_drops_the_queued_messages_then_is_delivered_itself() {
    let (server, handler, address) = start_server(Flavour::V1_0);
    let (mut client, _) = RawClient::handshake(address, Flavour::V1_0);
    handler.set_async_gate_closed(true);
    client.send_async(b"RUNNING");
    assert!(handler.wait_until(PROMPT, |events| {
        events.contains(&Event::Async(b"RUNNING".to_vec()))
    }));
    for index in 0..50 {
        client.send_async(format!("STALE\t{index}").as_bytes());
    }
    client.send_async(b"MISSIONEND");
    client.send_async(b"AFTER");
    // The session thread handles frames in order, so once this answer is back every async message
    // above is queued behind the held worker.
    assert!(matches!(
        client.sync_call(1, b"TS_INFO\tPING"),
        Frame::SyncResponse { id: 1, .. }
    ));
    handler.set_async_gate_closed(false);
    assert!(handler.wait_until(PROMPT, |events| {
        events.contains(&Event::Async(b"AFTER".to_vec()))
    }));
    thread::sleep(Duration::from_millis(50));
    assert_eq!(
        handler.async_texts(),
        [
            b"RUNNING".to_vec(),
            b"MISSIONEND".to_vec(),
            b"AFTER".to_vec()
        ]
    );
    server.shutdown();
}

#[test]
fn sync_and_async_messages_set_the_last_game_message_and_missionend_clears_it() {
    let (server, handler, address) = start_server(Flavour::V1_0);
    let (mut client, _) = RawClient::handshake(address, Flavour::V1_0);
    client.send(&Frame::Focus { arma_focused: true });
    assert!(handler.wait_until(PROMPT, |events| events.contains(&Event::Focus(true))));
    assert_eq!(
        server.last_game_message(),
        None,
        "FOCUS is not a game message"
    );

    client.sync_call(1, b"TS_INFO\tPING");
    let after_sync = server
        .last_game_message()
        .expect("SYNC_REQ is a game message");

    thread::sleep(Duration::from_millis(5));
    client.send_async(b"DFRAME");
    assert!(handler.wait_until(PROMPT, |events| {
        events.contains(&Event::Async(b"DFRAME".to_vec()))
    }));
    assert!(
        server.last_game_message().unwrap() > after_sync,
        "ASYNC is a game message"
    );

    client.send_async(b"MISSIONEND");
    assert!(handler.wait_until(PROMPT, |events| {
        events.contains(&Event::Async(b"MISSIONEND".to_vec()))
    }));
    assert_eq!(server.last_game_message(), None, "MISSIONEND resets it");
    server.shutdown();
}

#[test]
fn focus_frames_reach_the_handler_in_order() {
    let (server, handler, address) = start_server(Flavour::V1_0);
    let (mut client, _) = RawClient::handshake(address, Flavour::V1_0);
    for focused in [true, false, true] {
        client.send(&Frame::Focus {
            arma_focused: focused,
        });
    }
    assert!(handler.wait_until(PROMPT, |events| {
        events
            .iter()
            .filter(|event| matches!(event, Event::Focus(_)))
            .count()
            == 3
    }));
    let focus: Vec<Event> = handler
        .events()
        .into_iter()
        .filter(|event| matches!(event, Event::Focus(_)))
        .collect();
    assert_eq!(
        focus,
        [Event::Focus(true), Event::Focus(false), Event::Focus(true)]
    );
    server.shutdown();
}
