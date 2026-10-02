//! The `STATUS` heartbeat, the needcfg push and the shutdown of `GameLinkServer`, over real
//! loopback sockets.

// `cfg(test)` puts the helpers under the test lint relief of the root `clippy.toml`.
#[cfg(test)]
mod support;

use std::net::{Ipv4Addr, TcpListener};
use std::time::{Duration, Instant};

use support::{CloseOutcome, Event, PROMPT, RawClient, start_server, start_server_with};
use tfar_game_link_protocol::{Flavour, Frame};
use tfar_game_link_server::{GameLinkServer, ServerConfig};

/// The shutdown budget of the TeamSpeak plugin unload.
const SHUTDOWN_BUDGET: Duration = Duration::from_secs(1);

fn timed_shutdown(server: GameLinkServer, case: &str) -> Duration {
    let started = Instant::now();
    server.shutdown();
    let took = started.elapsed();
    eprintln!("shutdown ({case}) took {took:?}");
    assert!(took < SHUTDOWN_BUDGET, "shutdown ({case}) took {took:?}");
    took
}

#[test]
fn status_is_sent_every_500_milliseconds() {
    let (server, _handler, address) = start_server(Flavour::V1_0);
    let (mut client, _) = RawClient::handshake(address, Flavour::V1_0);
    let mut previous = Instant::now();
    let mut gaps = Vec::new();
    while gaps.len() < 4 {
        let frame = client.receive(Duration::from_secs(1)).expect("a heartbeat");
        assert!(
            matches!(frame, Frame::Status { needcfg: true }),
            "{frame:?}"
        );
        let now = Instant::now();
        gaps.push(now - previous);
        previous = now;
    }
    eprintln!("heartbeat gaps: {gaps:?}");
    for gap in &gaps {
        assert!(
            (Duration::from_millis(420)..=Duration::from_millis(620)).contains(gap),
            "heartbeat gap {gap:?}"
        );
    }
    server.shutdown();
}

#[test]
fn a_needcfg_change_is_pushed_at_once_and_an_unchanged_value_is_not() {
    let (server, _handler, address) = start_server(Flavour::V1_0);
    let (mut client, needcfg) = RawClient::handshake(address, Flavour::V1_0);
    assert!(needcfg);
    assert!(server.needcfg());

    let started = Instant::now();
    server.set_needcfg(false);
    assert_eq!(
        client.receive(Duration::from_millis(400)),
        Some(Frame::Status { needcfg: false })
    );
    let push_latency = started.elapsed();
    eprintln!("needcfg push latency: {push_latency:?}");
    assert!(push_latency < Duration::from_millis(100));

    // Right after a heartbeat the next one is about 500 ms away, so any frame sooner is a push.
    assert!(matches!(
        client.receive(Duration::from_secs(1)),
        Some(Frame::Status { needcfg: false })
    ));
    server.set_needcfg(false);
    assert_eq!(
        client.receive(Duration::from_millis(250)),
        None,
        "no push without a change"
    );

    server.set_needcfg(true);
    assert_eq!(
        client.receive(Duration::from_millis(100)),
        Some(Frame::Status { needcfg: true })
    );
    assert!(server.needcfg());
    server.shutdown();
}

#[test]
fn a_needcfg_set_before_the_session_is_carried_by_its_first_status() {
    let (server, _handler, address) = start_server(Flavour::V1_0);
    server.set_needcfg(false);
    let (_client, needcfg) = RawClient::handshake(address, Flavour::V1_0);
    assert!(!needcfg);
    server.shutdown();
}

#[test]
fn the_0_9_server_always_sends_needcfg_false() {
    let (server, _handler, address) = start_server(Flavour::V0_9);
    let (mut client, needcfg) = RawClient::handshake(address, Flavour::V0_9);
    assert!(!needcfg);
    server.set_needcfg(true);
    assert_eq!(
        client.receive(Duration::from_secs(1)),
        Some(Frame::Status { needcfg: false })
    );
    server.shutdown();
}

#[test]
fn shutdown_with_an_open_session_and_a_queue_finishes_under_one_second() {
    let (server, handler, address) = start_server(Flavour::V1_0);
    let (mut client, _) = RawClient::handshake(address, Flavour::V1_0);
    for index in 0..2000 {
        client.send(&Frame::Async {
            text: format!("POS\t{index}").into_bytes(),
        });
    }
    client.send(&Frame::SyncRequest {
        id: 1,
        text: b"TS_INFO\tPING".to_vec(),
    });
    assert!(client.receive_skipping_status(PROMPT).is_some());
    timed_shutdown(server, "open session, queued async messages");
    assert!(matches!(
        client.wait_for_close(PROMPT),
        CloseOutcome::Closed(_)
    ));
    assert_eq!(handler.session_events(), [Event::Opened, Event::Closed]);
}

#[test]
fn shutdown_with_a_connection_still_in_its_handshake_finishes_under_one_second() {
    let (server, handler, address) = start_server(Flavour::V1_0);
    let _silent = RawClient::connect(address);
    std::thread::sleep(Duration::from_millis(50));
    timed_shutdown(server, "connection awaiting HELLO");
    assert_eq!(handler.events(), Vec::<Event>::new());
}

#[test]
fn shutdown_while_the_bind_is_retried_finishes_under_one_second() {
    let blocker = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let port = blocker.local_addr().unwrap().port();
    let handler = support::RecordingHandler::new();
    let server = GameLinkServer::start(ServerConfig::new(Flavour::V1_0, port), handler).unwrap();
    assert_eq!(server.local_addr(), None);
    timed_shutdown(server, "bind retry pending");
}

#[test]
fn shutdown_with_no_connection_finishes_under_one_second_and_frees_the_port() {
    let (server, _handler, address) = start_server(Flavour::V0_9);
    timed_shutdown(server, "idle");
    TcpListener::bind(address).expect("the port is free after shutdown");
}

#[test]
fn dropping_the_server_shuts_it_down() {
    let (server, handler, address) = start_server_with(ServerConfig::new(Flavour::V1_0, 0));
    let (mut client, _) = RawClient::handshake(address, Flavour::V1_0);
    drop(server);
    assert!(matches!(
        client.wait_for_close(PROMPT),
        CloseOutcome::Closed(_)
    ));
    assert_eq!(handler.session_events(), [Event::Opened, Event::Closed]);
}
