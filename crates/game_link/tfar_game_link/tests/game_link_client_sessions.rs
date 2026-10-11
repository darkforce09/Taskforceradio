//! `GameLinkClient` against a real loopback listener: the handshake, sync round trips, timeouts,
//! closes, backoff, stale responses, the needcfg cache, dead sessions and refused connects, each
//! rule of the game link spec's extension side.

#[cfg(test)]
mod fake_plugin;

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use tfar_game_link::client::{ClientConfig, Error, GameLinkClient, SyncOutcome};
use tfar_game_link::protocol::{Flavour, Frame, Hello};

use fake_plugin::{FakePlugin, PluginConnection, free_port};

/// A client of `flavour` for the fake plugin's port, with the spec's timeouts.
fn client_for(plugin: &FakePlugin, flavour: Flavour) -> GameLinkClient {
    GameLinkClient::new(ClientConfig::new(flavour, plugin.port()))
}

#[test]
fn the_client_connects_shakes_hands_and_gets_the_sync_reply() {
    let plugin = FakePlugin::start(|mut connection: PluginConnection| {
        connection.accept_handshake(Flavour::V0_9, false);
        for expected_id in 1..=2 {
            let (id, text) = connection.read_sync_request();
            assert_eq!(id, expected_id, "request ids count up from 1 per session");
            assert_eq!(text, b"TS_INFO\tPING");
            connection.write(&Frame::SyncResponse {
                id,
                text: b"PONG".to_vec(),
            });
        }
        connection.hold_silently(Duration::from_secs(2));
    });
    let client = client_for(&plugin, Flavour::V0_9);
    assert!(!client.is_live(), "nothing connects before the first call");
    assert_eq!(plugin.accepts(), 0);

    assert_eq!(
        client.sync_request(b"TS_INFO\tPING"),
        SyncOutcome::Reply(b"PONG".to_vec())
    );
    assert!(client.is_live());
    assert_eq!(
        client.sync_request(b"TS_INFO\tPING"),
        SyncOutcome::Reply(b"PONG".to_vec())
    );
    assert_eq!(plugin.accepts(), 1, "both calls share one session");
}

#[test]
fn an_unanswered_sync_request_times_out_in_under_1_1_seconds() {
    let plugin = FakePlugin::start(|mut connection: PluginConnection| {
        connection.accept_handshake(Flavour::V0_9, false);
        connection.start_heartbeat(Duration::from_millis(200));
        let _ = connection.read_sync_request();
        connection.hold_silently(Duration::from_secs(3));
    });
    let client = client_for(&plugin, Flavour::V0_9);

    let started = Instant::now();
    assert_eq!(client.sync_request(b"TS_INFO\tPING"), SyncOutcome::TimedOut);
    let elapsed = started.elapsed();
    assert!(elapsed >= Duration::from_millis(1000), "waited {elapsed:?}");
    assert!(elapsed < Duration::from_millis(1100), "waited {elapsed:?}");
    assert!(client.is_live(), "a timeout keeps the session");
}

#[test]
fn the_plugin_closing_during_a_call_is_reported_as_closed_during_call() {
    let plugin = FakePlugin::start(|mut connection: PluginConnection| {
        connection.accept_handshake(Flavour::V0_9, false);
        let _ = connection.read_sync_request();
        drop(connection);
    });
    let client = client_for(&plugin, Flavour::V0_9);

    let started = Instant::now();
    assert_eq!(
        client.sync_request(b"TS_INFO\tPING"),
        SyncOutcome::ClosedDuringCall
    );
    assert!(started.elapsed() < Duration::from_millis(500));
    assert!(!client.is_live());
    assert_eq!(
        client.sync_request(b"TS_INFO\tPING"),
        SyncOutcome::NoSession,
        "a dropped session starts the backoff"
    );
}

#[test]
fn a_second_call_within_the_backoff_makes_no_new_connect_attempt() {
    // The plugin refuses every connection by closing it before HELLO.
    let plugin = FakePlugin::start(drop);
    let client = client_for(&plugin, Flavour::V0_9);

    assert_eq!(client.sync_request(b"PING"), SyncOutcome::NoSession);
    let started = Instant::now();
    assert_eq!(client.sync_request(b"PING"), SyncOutcome::NoSession);
    assert!(
        started.elapsed() < Duration::from_millis(50),
        "a call in the backoff answers at once"
    );
    assert!(matches!(
        client.send_async(b"PING"),
        Err(Error::BackoffRunning { .. })
    ));
    thread::sleep(Duration::from_millis(100));
    assert_eq!(plugin.accepts(), 1, "no connect while the backoff runs");

    thread::sleep(Duration::from_millis(1000));
    assert_eq!(client.sync_request(b"PING"), SyncOutcome::NoSession);
    thread::sleep(Duration::from_millis(100));
    assert_eq!(
        plugin.accepts(),
        2,
        "after the backoff the next call connects again"
    );
}

#[test]
fn a_response_with_an_older_id_is_discarded() {
    let plugin = FakePlugin::start(|mut connection: PluginConnection| {
        connection.accept_handshake(Flavour::V0_9, false);
        let (first_id, _) = connection.read_sync_request();
        let (second_id, _) = connection.read_sync_request();
        connection.write(&Frame::SyncResponse {
            id: first_id,
            text: b"STALE".to_vec(),
        });
        connection.write(&Frame::SyncResponse {
            id: second_id,
            text: b"FRESH".to_vec(),
        });
        connection.hold_silently(Duration::from_secs(2));
    });
    let mut config = ClientConfig::new(Flavour::V0_9, plugin.port());
    config.sync_timeout = Duration::from_millis(200);
    let client = GameLinkClient::new(config);

    assert_eq!(client.sync_request(b"FIRST"), SyncOutcome::TimedOut);
    assert_eq!(
        client.sync_request(b"SECOND"),
        SyncOutcome::Reply(b"FRESH".to_vec())
    );
}

#[test]
fn needcfg_follows_the_latest_status_from_the_plugin() {
    let plugin = FakePlugin::start(|mut connection: PluginConnection| {
        connection.accept_handshake(Flavour::V1_0, true);
        for needcfg in [true, false, true] {
            let (id, _) = connection.read_sync_request();
            connection.write(&Frame::Status { needcfg });
            connection.write(&Frame::SyncResponse {
                id,
                text: b"OK".to_vec(),
            });
        }
        connection.hold_silently(Duration::from_secs(2));
    });
    let client = client_for(&plugin, Flavour::V1_0);
    assert!(client.needcfg(), "needcfg is true before any STATUS");

    for expected in [true, false, true] {
        assert_eq!(
            client.sync_request(b"POS"),
            SyncOutcome::Reply(b"OK".to_vec())
        );
        assert_eq!(client.needcfg(), expected);
    }
}

#[test]
fn a_session_without_plugin_frames_for_the_dead_time_is_dropped() {
    let plugin = FakePlugin::start(|mut connection: PluginConnection| {
        connection.accept_handshake(Flavour::V1_0, false);
        connection.hold_silently(Duration::from_secs(3));
    });
    let mut config = ClientConfig::new(Flavour::V1_0, plugin.port());
    config.session_dead_after = Duration::from_millis(300);
    let client = GameLinkClient::new(config);

    client.send_async(b"DFRAME").unwrap();
    assert!(client.is_live());
    thread::sleep(Duration::from_millis(400));
    assert!(
        !client.is_live(),
        "no plugin frame for longer than the dead time"
    );

    let started = Instant::now();
    assert_eq!(client.sync_request(b"POS"), SyncOutcome::NoSession);
    assert!(
        started.elapsed() < Duration::from_millis(50),
        "a dead session is dropped without a wait"
    );
    assert_eq!(plugin.accepts(), 1, "the drop starts the backoff");
}

#[test]
fn heartbeats_keep_a_quiet_session_live() {
    let plugin = FakePlugin::start(|mut connection: PluginConnection| {
        connection.accept_handshake(Flavour::V1_0, false);
        connection.start_heartbeat(Duration::from_millis(100));
        connection.hold_silently(Duration::from_secs(3));
    });
    let mut config = ClientConfig::new(Flavour::V1_0, plugin.port());
    config.session_dead_after = Duration::from_millis(300);
    let client = GameLinkClient::new(config);

    client.send_async(b"DFRAME").unwrap();
    thread::sleep(Duration::from_millis(600));
    assert!(client.is_live());
}

#[test]
fn a_port_nobody_listens_on_fails_within_the_connect_timeout() {
    let config = ClientConfig::new(Flavour::V0_9, free_port());
    let client = GameLinkClient::new(config);

    let started = Instant::now();
    assert_eq!(
        client.sync_request(b"TS_INFO\tPING"),
        SyncOutcome::NoSession
    );
    let elapsed = started.elapsed();
    assert!(elapsed <= config.connect_timeout, "took {elapsed:?}");
}

#[test]
fn a_plugin_hello_of_the_other_flavour_opens_no_session() {
    let plugin = FakePlugin::start(|mut connection: PluginConnection| {
        let _ = connection.read();
        connection.write(&Frame::Hello(Hello::new(Flavour::V0_9)));
        connection.hold_silently(Duration::from_secs(2));
    });
    let client = client_for(&plugin, Flavour::V1_0);

    assert!(matches!(
        client.send_async(b"DFRAME"),
        Err(Error::HelloRefused { .. })
    ));
    assert!(!client.is_live());
}

#[test]
fn a_plugin_that_never_says_hello_fails_within_the_hello_timeout() {
    let plugin = FakePlugin::start(|connection: PluginConnection| {
        connection.hold_silently(Duration::from_secs(2));
    });
    let client = client_for(&plugin, Flavour::V0_9);

    let started = Instant::now();
    assert_eq!(client.sync_request(b"PING"), SyncOutcome::NoSession);
    let elapsed = started.elapsed();
    assert!(elapsed >= Duration::from_millis(250), "took {elapsed:?}");
    assert!(elapsed < Duration::from_millis(400), "took {elapsed:?}");
}

#[test]
fn async_and_focus_frames_reach_the_plugin_in_order() {
    let (frames_sender, frames) = mpsc::channel();
    let plugin = FakePlugin::start(move |mut connection: PluginConnection| {
        connection.accept_handshake(Flavour::V1_0, false);
        for _ in 0..3 {
            frames_sender.send(connection.read()).unwrap();
        }
        connection.hold_silently(Duration::from_secs(2));
    });
    let client = client_for(&plugin, Flavour::V1_0);

    client.send_async(b"DFRAME").unwrap();
    client.send_focus(false).unwrap();
    client.send_async(b"MISSIONEND").unwrap();

    let timeout = Duration::from_secs(2);
    assert_eq!(
        frames.recv_timeout(timeout).unwrap(),
        Frame::Async {
            text: b"DFRAME".to_vec()
        }
    );
    assert_eq!(
        frames.recv_timeout(timeout).unwrap(),
        Frame::Focus {
            arma_focused: false
        }
    );
    assert_eq!(
        frames.recv_timeout(timeout).unwrap(),
        Frame::Async {
            text: b"MISSIONEND".to_vec()
        }
    );
}

#[test]
fn a_new_session_replays_the_latest_focus_right_after_the_handshake() {
    let (frames_sender, frames) = mpsc::channel();
    let plugin = FakePlugin::start(move |mut connection: PluginConnection| {
        connection.accept_handshake(Flavour::V1_0, false);
        let first = connection.read();
        frames_sender.send((connection.index, first)).unwrap();
        if connection.index == 0 {
            // The first session ends here; the client must replay the focus on the next one.
            return;
        }
        let second = connection.read();
        frames_sender.send((connection.index, second)).unwrap();
        connection.hold_silently(Duration::from_secs(2));
    });
    let mut config = ClientConfig::new(Flavour::V1_0, plugin.port());
    config.backoff = Duration::from_millis(100);
    let client = GameLinkClient::new(config);
    let timeout = Duration::from_secs(2);

    client.send_focus(true).unwrap();
    assert_eq!(
        frames.recv_timeout(timeout).unwrap(),
        (0, Frame::Focus { arma_focused: true }),
        "a session opened by send_focus carries the value once"
    );

    thread::sleep(Duration::from_millis(200));
    client.send_async(b"DFRAME").unwrap();
    assert_eq!(
        frames.recv_timeout(timeout).unwrap(),
        (1, Frame::Focus { arma_focused: true })
    );
    assert_eq!(
        frames.recv_timeout(timeout).unwrap(),
        (
            1,
            Frame::Async {
                text: b"DFRAME".to_vec()
            }
        )
    );
}
