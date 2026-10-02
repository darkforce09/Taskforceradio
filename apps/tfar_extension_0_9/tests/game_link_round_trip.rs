//! The 0.9 extension's call handling over the real `GameLinkClient`, against a real
//! `GameLinkServer` and a raw-frame stand-in plugin, each on an ephemeral loopback port.

use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

use tfar_extension_0_9::prelude::*;
use tfar_game_link_client::prelude::{ClientConfig, Flavour, GameLinkClient};
use tfar_game_link_protocol::{Frame, read_frame, write_frame};

use stand_in_plugin::{client_for, closed_port, start_server};

#[cfg(test)]
mod stand_in_plugin {
    use std::net::TcpListener;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    use tfar_game_link_client::prelude::{ClientConfig, Flavour, GameLinkClient};
    use tfar_game_link_server::{GameLinkHandler, GameLinkServer, ServerConfig};

    /// A plugin stand-in: answers `TS_INFO\tPING` with `PONG`, sleeps on `SLOW`, echoes the rest.
    struct PingHandler;

    impl GameLinkHandler for PingHandler {
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

        fn async_message(&self, _text: &[u8]) {}

        fn focus_changed(&self, _focused: bool) {}

        fn session_opened(&self) {}

        fn session_closed(&self) {}
    }

    /// A 0.9 server on an ephemeral port, and its port.
    pub(crate) fn start_server() -> (GameLinkServer, u16) {
        let server =
            GameLinkServer::start(ServerConfig::new(Flavour::V0_9, 0), Arc::new(PingHandler))
                .unwrap();
        let port = server.local_addr().unwrap().port();
        (server, port)
    }

    /// A client for `port` with a short sync wait, so the timeout test stays quick.
    pub(crate) fn client_for(port: u16) -> GameLinkClient {
        let mut config = ClientConfig::new(Flavour::V0_9, port);
        config.sync_timeout = Duration::from_millis(150);
        GameLinkClient::new(config)
    }

    /// A port nothing listens on: bound once, then released.
    pub(crate) fn closed_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }
}

#[test]
fn ping_through_the_real_server_replies_pong() {
    let (server, port) = start_server();
    let client = client_for(port);
    assert_eq!(answer_call(&client, "TS_INFO\tPING"), "PONG");
    assert_eq!(answer_call(&client, "TRACK\tsome text"), "TRACK\tsome text");
    assert_eq!(answer_call(&client, ""), "");
    server.shutdown();
}

#[test]
fn no_plugin_replies_pipe_not_opened_and_the_backoff_keeps_it() {
    let client = client_for(closed_port());
    let started = Instant::now();
    assert_eq!(answer_call(&client, "TS_INFO\tPING"), PIPE_NOT_OPENED);
    assert_eq!(answer_call(&client, "TS_INFO\tPING"), PIPE_NOT_OPENED);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn a_slow_plugin_replies_not_connected_to_teamspeak() {
    let (server, port) = start_server();
    let client = client_for(port);
    assert_eq!(answer_call(&client, "SLOW"), NOT_CONNECTED_TO_TEAMSPEAK);
    server.shutdown();
}

#[test]
fn a_plugin_that_closes_during_the_call_replies_pipe_closed_by_ts() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let plugin = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        assert!(matches!(read_frame(&mut stream).unwrap(), Frame::Hello(_)));
        write_frame(&mut stream, &Frame::hello(Flavour::V0_9)).unwrap();
        write_frame(&mut stream, &Frame::Status { needcfg: false }).unwrap();
        let request = read_frame(&mut stream).unwrap();
        assert!(matches!(request, Frame::SyncRequest { .. }));
        drop(stream);
    });
    let mut config = ClientConfig::new(Flavour::V0_9, port);
    config.sync_timeout = Duration::from_secs(2);
    let client = GameLinkClient::new(config);
    assert_eq!(
        answer_call(&client, "TS_INFO\tPING"),
        PIPE_CLOSED_BY_TEAMSPEAK
    );
    plugin.join().unwrap();
}
