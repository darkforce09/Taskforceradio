//! The two listen ports of `GameLinkServer`: a `HELLO` on either port opens the one session, a
//! `HELLO` on the other port replaces it, and a busy debug port leaves the normal one working.

// `cfg(test)` puts the helpers under the test lint relief of the root `clippy.toml`.
#[cfg(test)]
mod support;

use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::thread;
use std::time::{Duration, Instant};

use support::{CloseOutcome, Event, PROMPT, RawClient, RecordingHandler};
use tfar_game_link::protocol::{Flavour, Frame};
use tfar_game_link::server::{GameLinkServer, ServerConfig};

/// A server on two ephemeral ports, with its handler and the two bound addresses, normal first.
#[cfg(test)]
fn start_two_port_server(
    flavour: Flavour,
) -> (
    GameLinkServer,
    std::sync::Arc<RecordingHandler>,
    [SocketAddr; 2],
) {
    let handler = RecordingHandler::new();
    let config = ServerConfig::new(flavour, 0).with_debug_port(0);
    let server = GameLinkServer::start(config, handler.clone()).unwrap();
    let addresses = server.local_addrs();
    let [normal, debug] = addresses[..] else {
        panic!("two ephemeral ports bind at once, got {addresses:?}");
    };
    assert_ne!(normal.port(), debug.port());
    assert_eq!(server.local_addr(), Some(normal));
    (server, handler, [normal, debug])
}

/// The session callbacks among `events`, in order.
fn session_events_of(events: &[Event]) -> Vec<Event> {
    events
        .iter()
        .filter(|event| matches!(event, Event::Opened | Event::Closed))
        .cloned()
        .collect()
}

/// Waits until `server` reports `up`, up to [`PROMPT`].
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
fn both_ports_are_bound_on_loopback_only() {
    let (server, _handler, addresses) = start_two_port_server(Flavour::V1_0);
    for address in addresses {
        assert_eq!(address.ip(), Ipv4Addr::LOCALHOST);
    }
    server.shutdown();
}

#[test]
fn a_hello_on_each_port_opens_a_session_that_answers() {
    let (server, handler, addresses) = start_two_port_server(Flavour::V1_0);
    for (index, address) in addresses.into_iter().enumerate() {
        let (mut client, needcfg) = RawClient::handshake(address, Flavour::V1_0);
        assert!(needcfg, "the 1.0 server starts with needcfg");
        assert!(wait_for_session_up(&server, true), "port {index}");
        let id = u32::try_from(index).unwrap();
        assert_eq!(
            client.sync_call(id, b"TS_INFO\tPING"),
            Frame::SyncResponse {
                id,
                text: b"PONG".to_vec()
            },
            "port {index}"
        );
        drop(client);
        assert!(wait_for_session_up(&server, false), "port {index}");
    }
    assert!(handler.wait_until(PROMPT, |events| {
        events
            .iter()
            .filter(|event| matches!(event, Event::Opened | Event::Closed))
            .count()
            == 4
    }));
    assert_eq!(
        handler.session_events(),
        [Event::Opened, Event::Closed, Event::Opened, Event::Closed]
    );
    server.shutdown();
}

#[test]
fn a_hello_on_the_other_port_replaces_the_open_session() {
    let (server, handler, [normal, debug]) = start_two_port_server(Flavour::V0_9);
    let (mut first, _) = RawClient::handshake(normal, Flavour::V0_9);
    assert!(handler.wait_until(PROMPT, |events| events == [Event::Opened]));
    let (mut second, _) = RawClient::handshake(debug, Flavour::V0_9);
    assert!(handler.wait_until(PROMPT, |events| {
        events == [Event::Opened, Event::Closed, Event::Opened]
    }));
    assert!(
        matches!(first.wait_for_close(PROMPT), CloseOutcome::Closed(_)),
        "the replaced session's socket is closed"
    );
    assert!(server.is_session_up());
    assert_eq!(
        second.sync_call(3, b"TS_INFO\tPING"),
        Frame::SyncResponse {
            id: 3,
            text: b"PONG".to_vec()
        }
    );

    let (mut third, _) = RawClient::handshake(normal, Flavour::V0_9);
    assert!(handler.wait_until(PROMPT, |events| {
        session_events_of(events)
            == [
                Event::Opened,
                Event::Closed,
                Event::Opened,
                Event::Closed,
                Event::Opened,
            ]
    }));
    assert!(matches!(
        second.wait_for_close(PROMPT),
        CloseOutcome::Closed(_)
    ));
    assert!(matches!(
        third.sync_call(4, b"TS_INFO\tPING"),
        Frame::SyncResponse { id: 4, .. }
    ));
    server.shutdown();
}

#[test]
fn a_refused_hello_on_the_debug_port_leaves_the_normal_session_up() {
    let (server, handler, [normal, debug]) = start_two_port_server(Flavour::V1_0);
    let (mut client, _) = RawClient::handshake(normal, Flavour::V1_0);
    assert!(handler.wait_until(PROMPT, |events| events == [Event::Opened]));
    let mut intruder = RawClient::connect(debug);
    intruder.send(&Frame::hello(Flavour::V0_9));
    assert_eq!(
        intruder.wait_for_close(PROMPT),
        CloseOutcome::Closed(vec![])
    );
    assert_eq!(handler.session_events(), [Event::Opened]);
    assert!(matches!(
        client.sync_call(1, b"TS_INFO\tPING"),
        Frame::SyncResponse { id: 1, .. }
    ));
    server.shutdown();
}

#[test]
fn a_busy_debug_port_leaves_the_normal_port_working_and_is_retried_on_its_own() {
    let blocker = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let busy_port = blocker.local_addr().unwrap().port();
    let config = ServerConfig {
        bind_retry_interval: Duration::from_millis(100),
        ..ServerConfig::new(Flavour::V1_0, 0).with_debug_port(busy_port)
    };
    let handler = RecordingHandler::new();
    let server = GameLinkServer::start(config, handler.clone()).unwrap();
    let normal = server
        .local_addr()
        .expect("the normal port binds although the debug port is busy");
    assert_eq!(server.local_addrs(), [normal], "the debug port is dormant");

    let (mut client, _) = RawClient::handshake(normal, Flavour::V1_0);
    assert!(handler.wait_until(PROMPT, |events| events == [Event::Opened]));
    assert!(matches!(
        client.sync_call(1, b"TS_INFO\tPING"),
        Frame::SyncResponse { id: 1, .. }
    ));
    thread::sleep(Duration::from_millis(250));
    assert_eq!(server.local_addrs(), [normal]);

    drop(blocker);
    let deadline = Instant::now() + PROMPT;
    while server.local_addrs().len() < 2 {
        assert!(
            Instant::now() < deadline,
            "the debug port's retry never bound the freed port"
        );
        thread::sleep(Duration::from_millis(10));
    }
    let debug = *server.local_addrs().last().unwrap();
    assert_eq!(debug.port(), busy_port);
    assert!(
        server.is_session_up(),
        "binding the debug port leaves the open session alone"
    );
    let (_debug_client, _) = RawClient::handshake(debug, Flavour::V1_0);
    assert!(handler.wait_until(PROMPT, |events| {
        session_events_of(events) == [Event::Opened, Event::Closed, Event::Opened]
    }));
    server.shutdown();
}

#[test]
fn a_busy_normal_port_leaves_the_debug_port_working() {
    let blocker = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let busy_port = blocker.local_addr().unwrap().port();
    let handler = RecordingHandler::new();
    let config = ServerConfig::new(Flavour::V0_9, busy_port).with_debug_port(0);
    let server = GameLinkServer::start(config, handler.clone()).unwrap();
    assert_eq!(server.local_addr(), None, "the normal port is dormant");
    let addresses = server.local_addrs();
    let [debug] = addresses[..] else {
        panic!("the debug port binds alone, got {addresses:?}");
    };
    let (_client, needcfg) = RawClient::handshake(debug, Flavour::V0_9);
    assert!(!needcfg);
    assert!(handler.wait_until(PROMPT, |events| events == [Event::Opened]));
    server.shutdown();
    drop(blocker);
}

#[test]
fn shutdown_of_a_two_port_server_frees_both_ports() {
    let (server, _handler, addresses) = start_two_port_server(Flavour::V1_0);
    let started = Instant::now();
    server.shutdown();
    assert!(started.elapsed() < Duration::from_secs(1));
    for address in addresses {
        TcpListener::bind(address).expect("the port is free after shutdown");
    }
}
