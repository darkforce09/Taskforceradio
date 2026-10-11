//! The handshake, the session lifecycle and the listener of `GameLinkServer`, over real loopback
//! sockets.

// `cfg(test)` puts the helpers under the test lint relief of the root `clippy.toml`.
#[cfg(test)]
mod support;

use std::net::{Ipv4Addr, TcpListener};
use std::thread;
use std::time::{Duration, Instant};

use support::{CloseOutcome, Event, PROMPT, RawClient, start_server, start_server_with};
use tfar_game_link::protocol::{Flavour, Frame, Hello, MAX_FRAME_LENGTH};
use tfar_game_link::server::{GameLinkServer, ServerConfig};

fn wait_for_session_up(server: &GameLinkServer, up: bool) -> bool {
    let deadline = Instant::now() + PROMPT;
    while server.is_session_up() != up {
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(5));
    }
    true
}

#[test]
fn the_server_listens_on_loopback_only() {
    let (server, _handler, address) = start_server(Flavour::V1_0);
    assert_eq!(address.ip(), Ipv4Addr::LOCALHOST);
    assert_ne!(address.port(), 0);
    server.shutdown();
}

#[test]
fn a_valid_hello_is_answered_with_hello_then_status_and_opens_the_session() {
    for flavour in [Flavour::V0_9, Flavour::V1_0] {
        let (server, handler, address) = start_server(flavour);
        assert!(!server.is_session_up());
        let (_client, needcfg) = RawClient::handshake(address, flavour);
        assert_eq!(
            needcfg,
            flavour == Flavour::V1_0,
            "needcfg starts true for 1.0 only"
        );
        assert!(handler.wait_until(PROMPT, |events| events == [Event::Opened]));
        assert!(server.is_session_up());
        assert_eq!(
            server.last_game_message(),
            None,
            "HELLO is not a game message"
        );
        server.shutdown();
    }
}

fn assert_hello_refused_without_reply(hello: Hello) {
    let (server, handler, address) = start_server(Flavour::V1_0);
    let mut client = RawClient::connect(address);
    client.send(&Frame::Hello(hello));
    assert_eq!(
        client.wait_for_close(PROMPT),
        CloseOutcome::Closed(Vec::new()),
        "a refused HELLO gets no reply"
    );
    assert!(!server.is_session_up());
    assert_eq!(handler.events(), Vec::<Event>::new());
    server.shutdown();
}

#[test]
fn a_hello_with_another_magic_is_closed_without_a_reply() {
    assert_hello_refused_without_reply(Hello {
        magic: *b"TFAX",
        ..Hello::new(Flavour::V1_0)
    });
}

#[test]
fn a_hello_with_another_protocol_is_closed_without_a_reply() {
    assert_hello_refused_without_reply(Hello {
        protocol: 2,
        ..Hello::new(Flavour::V1_0)
    });
}

#[test]
fn a_hello_of_the_other_flavour_is_closed_without_a_reply() {
    assert_hello_refused_without_reply(Hello::new(Flavour::V0_9));
    assert_hello_refused_without_reply(Hello {
        flavour: 11,
        ..Hello::new(Flavour::V1_0)
    });
}

#[test]
fn a_first_frame_other_than_hello_is_closed_without_a_reply() {
    let (server, handler, address) = start_server(Flavour::V1_0);
    let mut client = RawClient::connect(address);
    client.send(&Frame::SyncRequest {
        id: 1,
        text: b"TS_INFO\tPING".to_vec(),
    });
    assert_eq!(
        client.wait_for_close(PROMPT),
        CloseOutcome::Closed(Vec::new())
    );
    assert_eq!(handler.events(), Vec::<Event>::new());
    server.shutdown();
}

#[test]
fn a_connection_that_sends_no_hello_is_dropped_after_the_hello_timeout() {
    let config = ServerConfig {
        hello_timeout: Duration::from_millis(200),
        ..ServerConfig::new(Flavour::V1_0, 0)
    };
    let (server, handler, address) = start_server_with(config);
    let mut client = RawClient::connect(address);
    let started = Instant::now();
    assert_eq!(
        client.wait_for_close(PROMPT),
        CloseOutcome::Closed(Vec::new())
    );
    assert!(started.elapsed() >= Duration::from_millis(150));
    assert_eq!(handler.events(), Vec::<Event>::new());
    server.shutdown();
}

#[test]
fn a_new_valid_hello_replaces_the_open_session() {
    let (server, handler, address) = start_server(Flavour::V1_0);
    let (mut first, _) = RawClient::handshake(address, Flavour::V1_0);
    assert!(handler.wait_until(PROMPT, |events| events == [Event::Opened]));
    let (mut second, _) = RawClient::handshake(address, Flavour::V1_0);
    assert!(matches!(
        first.wait_for_close(PROMPT),
        CloseOutcome::Closed(_)
    ));
    assert!(handler.wait_until(PROMPT, |events| {
        events == [Event::Opened, Event::Closed, Event::Opened]
    }));
    assert!(server.is_session_up());
    second.send(&Frame::SyncRequest {
        id: 1,
        text: b"TS_INFO\tPING".to_vec(),
    });
    assert_eq!(
        second.receive_skipping_status(PROMPT),
        Some(Frame::SyncResponse {
            id: 1,
            text: b"PONG".to_vec()
        })
    );
    server.shutdown();
    assert_eq!(
        handler.session_events(),
        [Event::Opened, Event::Closed, Event::Opened, Event::Closed]
    );
}

#[test]
fn a_refused_hello_leaves_the_open_session_up() {
    let (server, handler, address) = start_server(Flavour::V1_0);
    let (mut first, _) = RawClient::handshake(address, Flavour::V1_0);
    let mut intruder = RawClient::connect(address);
    intruder.send(&Frame::hello(Flavour::V0_9));
    assert_eq!(
        intruder.wait_for_close(PROMPT),
        CloseOutcome::Closed(Vec::new())
    );
    assert!(server.is_session_up());
    first.send(&Frame::SyncRequest {
        id: 1,
        text: b"STILL".to_vec(),
    });
    assert_eq!(
        first.receive_skipping_status(PROMPT),
        Some(Frame::SyncResponse {
            id: 1,
            text: b"ECHO STILL".to_vec()
        })
    );
    assert_eq!(handler.session_events(), [Event::Opened]);
    server.shutdown();
}

#[test]
fn the_peer_closing_the_connection_closes_the_session() {
    let (server, handler, address) = start_server(Flavour::V0_9);
    let (client, _) = RawClient::handshake(address, Flavour::V0_9);
    assert!(wait_for_session_up(&server, true));
    drop(client);
    assert!(handler.wait_until(PROMPT, |events| events == [Event::Opened, Event::Closed]));
    assert!(wait_for_session_up(&server, false));
    server.shutdown();
}

fn assert_malformed_bytes_close_the_session(bytes: &[u8]) {
    let (server, handler, address) = start_server(Flavour::V1_0);
    let (mut client, _) = RawClient::handshake(address, Flavour::V1_0);
    client.send_raw(bytes);
    let outcome = client.wait_for_close(PROMPT);
    assert!(
        matches!(outcome, CloseOutcome::Closed(_)),
        "the session stays open after {bytes:02x?}"
    );
    let CloseOutcome::Closed(frames) = outcome else {
        return;
    };
    assert!(
        frames
            .iter()
            .all(|frame| matches!(frame, Frame::Status { .. })),
        "only heartbeats before the close: {frames:?}"
    );
    assert!(handler.wait_until(PROMPT, |events| events == [Event::Opened, Event::Closed]));
    assert!(wait_for_session_up(&server, false));
    server.shutdown();
}

#[test]
fn a_zero_length_frame_closes_the_session() {
    assert_malformed_bytes_close_the_session(&[0, 0, 0, 0]);
}

#[test]
fn a_frame_over_the_length_limit_closes_the_session() {
    assert_malformed_bytes_close_the_session(&(MAX_FRAME_LENGTH + 1).to_le_bytes());
}

#[test]
fn an_unknown_frame_kind_closes_the_session() {
    assert_malformed_bytes_close_the_session(&[1, 0, 0, 0, 0x09]);
}

#[test]
fn a_payload_that_does_not_decode_closes_the_session() {
    assert_malformed_bytes_close_the_session(&[2, 0, 0, 0, 0x05, 0x07]);
}

#[test]
fn a_busy_port_is_retried_until_it_frees_up() {
    let blocker = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let port = blocker.local_addr().unwrap().port();
    let config = ServerConfig {
        bind_retry_interval: Duration::from_millis(100),
        ..ServerConfig::new(Flavour::V1_0, port)
    };
    let handler = support::RecordingHandler::new();
    let server = GameLinkServer::start(config, handler.clone()).unwrap();
    assert_eq!(
        server.local_addr(),
        None,
        "the port is busy, so the server is dormant"
    );
    thread::sleep(Duration::from_millis(250));
    assert_eq!(server.local_addr(), None);
    drop(blocker);
    let deadline = Instant::now() + PROMPT;
    while server.local_addr().is_none() {
        assert!(
            Instant::now() < deadline,
            "the retry never bound the freed port"
        );
        thread::sleep(Duration::from_millis(10));
    }
    let address = server.local_addr().unwrap();
    assert_eq!(address.port(), port);
    let (_client, _) = RawClient::handshake(address, Flavour::V1_0);
    assert!(handler.wait_until(PROMPT, |events| events == [Event::Opened]));
    server.shutdown();
}
