//! The 1.0 extension's call handling over the real `GameLinkClient`, against a real
//! `GameLinkServer` on an ephemeral loopback port: sync replies, async replies with the needcfg
//! value, `MISSIONEND`, the `FOCUS` frame, a missing plugin and a sync timeout.

use std::time::{Duration, Instant};

use tfar_extension_1_0::prelude::*;

use recording_plugin::{Event, client_for, closed_port, start_server};

#[cfg(test)]
mod recording_plugin {
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    use tfar_game_link_client::prelude::{ClientConfig, Flavour, GameLinkClient};
    use tfar_game_link_server::{GameLinkHandler, GameLinkServer, ServerConfig};

    /// How long a test waits for something the server should see at once.
    const PROMPT: Duration = Duration::from_secs(2);

    /// One callback the server made.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(crate) enum Event {
        Async(Vec<u8>),
        Focus(bool),
    }

    /// A plugin stand-in that answers `TS_INFO\tPING` with `PONG`, sleeps on `SLOW` and records
    /// every async message and focus change.
    #[derive(Default)]
    pub(crate) struct RecordingHandler {
        events: Mutex<Vec<Event>>,
    }

    impl RecordingHandler {
        pub(crate) fn events(&self) -> Vec<Event> {
            self.events.lock().unwrap().clone()
        }

        /// Waits until `count` events are recorded, then returns them.
        pub(crate) fn wait_for_events(&self, count: usize) -> Vec<Event> {
            let deadline = Instant::now() + PROMPT;
            while self.events().len() < count && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(5));
            }
            self.events()
        }
    }

    impl GameLinkHandler for RecordingHandler {
        fn sync_request(&self, text: &[u8]) -> Vec<u8> {
            match text {
                b"TS_INFO\tPING" => b"PONG".to_vec(),
                b"SLOW" => {
                    thread::sleep(Duration::from_millis(400));
                    b"LATE".to_vec()
                }
                other => other.to_vec(),
            }
        }

        fn async_message(&self, text: &[u8]) {
            self.events
                .lock()
                .unwrap()
                .push(Event::Async(text.to_vec()));
        }

        fn focus_changed(&self, focused: bool) {
            self.events.lock().unwrap().push(Event::Focus(focused));
        }

        fn session_opened(&self) {}

        fn session_closed(&self) {}
    }

    /// A 1.0 server on an ephemeral port with its recording handler, and its port.
    pub(crate) fn start_server() -> (GameLinkServer, Arc<RecordingHandler>, u16) {
        let handler = Arc::new(RecordingHandler::default());
        let server = GameLinkServer::start(
            ServerConfig::new(Flavour::V1_0, 0),
            Arc::clone(&handler) as Arc<dyn GameLinkHandler>,
        )
        .unwrap();
        let port = server.local_addr().unwrap().port();
        (server, handler, port)
    }

    /// A client for `port` with a short sync wait, so the timeout test stays quick.
    pub(crate) fn client_for(port: u16) -> GameLinkClient {
        let mut config = ClientConfig::new(Flavour::V1_0, port);
        config.sync_timeout = Duration::from_millis(150);
        GameLinkClient::new(config)
    }

    /// A port nothing listens on: bound once, then released.
    pub(crate) fn closed_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }

    /// Waits until `condition` holds or the prompt time passes; returns whether it held.
    pub(crate) fn eventually(condition: impl Fn() -> bool) -> bool {
        let deadline = Instant::now() + PROMPT;
        while !condition() {
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(5));
        }
        true
    }
}

#[test]
fn sync_async_mission_end_and_focus_through_the_real_server() {
    let (server, handler, port) = start_server();
    let client = client_for(port);
    let mut focus = FocusWatch::new();
    let start = Instant::now();

    let reply = answer_call(&client, &mut focus, start, || true, "TS_INFO\tPING");
    assert_eq!(reply, "PONG");
    assert_eq!(handler.wait_for_events(1), vec![Event::Focus(true)]);

    // The 1.0 plugin starts with needcfg, so the first data frame asks for the settings.
    let reply = answer_call(&client, &mut focus, start, || true, "DFRAME~");
    assert_eq!(reply, CONFIGURATION_NEEDED);
    let reply = answer_call(
        &client,
        &mut focus,
        start,
        || true,
        "SETCFG\tpluginTimeout\t4~",
    );
    assert_eq!(reply, ASYNC_ACCEPTED);
    handler.wait_for_events(3);
    server.set_needcfg(false);
    assert!(recording_plugin::eventually(|| !client.needcfg()));
    let reply = answer_call(&client, &mut focus, start, || true, "DFRAME~");
    assert_eq!(reply, ASYNC_ACCEPTED);
    handler.wait_for_events(4);

    let later = start + FOCUS_READ_INTERVAL;
    let reply = answer_call(&client, &mut focus, later, || false, "MISSIONEND~");
    assert_eq!(reply, "");
    assert_eq!(
        handler.wait_for_events(6),
        vec![
            Event::Focus(true),
            Event::Async(b"DFRAME".to_vec()),
            Event::Async(b"SETCFG\tpluginTimeout\t4".to_vec()),
            Event::Async(b"DFRAME".to_vec()),
            Event::Focus(false),
            Event::Async(b"MISSIONEND".to_vec()),
        ]
    );
    assert_eq!(answer_call(&client, &mut focus, later, || false, ""), "");
    server.shutdown();
}

#[test]
fn no_plugin_replies_not_connected_to_teamspeak_for_every_call() {
    let client = client_for(closed_port());
    let mut focus = FocusWatch::new();
    let started = Instant::now();
    for call in ["TS_INFO\tPING", "DFRAME~", "MISSIONEND~", "SETCFG\ta\tb~"] {
        let reply = answer_call(&client, &mut focus, Instant::now(), || true, call);
        assert_eq!(reply, NOT_CONNECTED_TO_TEAMSPEAK, "for {call:?}");
    }
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn a_slow_plugin_replies_empty() {
    let (server, _handler, port) = start_server();
    let client = client_for(port);
    let reply = answer_call(
        &client,
        &mut FocusWatch::new(),
        Instant::now(),
        || false,
        "SLOW",
    );
    assert_eq!(reply, "");
    server.shutdown();
}
