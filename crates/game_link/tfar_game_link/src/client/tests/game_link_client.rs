//! Unit tests for `GameLinkClient` that need no plugin; the socket scenarios live in the crate's
//! `tests/` folder.

use std::net::TcpListener;
use std::time::{Duration, Instant};

use crate::protocol::Flavour;

use super::*;

/// A port nothing listens on: bound once to get a free port, then released.
fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

#[test]
fn a_new_client_is_not_live_and_asks_for_the_configuration() {
    let client = GameLinkClient::new(ClientConfig::new(Flavour::V1_0, free_port()));
    assert!(!client.is_live());
    assert!(client.needcfg());
}

#[test]
fn a_failed_connect_starts_the_backoff() {
    let client = GameLinkClient::new(ClientConfig::new(Flavour::V0_9, free_port()));
    assert!(matches!(
        client.send_async(b"X"),
        Err(Error::Connect { .. })
    ));
    let started = Instant::now();
    assert!(matches!(
        client.send_async(b"X"),
        Err(Error::BackoffRunning { remaining }) if remaining <= Duration::from_secs(1)
    ));
    assert!(started.elapsed() < Duration::from_millis(50));
}

#[test]
fn send_focus_keeps_the_value_even_when_no_session_opens() {
    let client = GameLinkClient::new(ClientConfig::new(Flavour::V1_0, free_port()));
    assert!(client.send_focus(true).is_err());
    assert_eq!(client.lock_connection().last_focus, Some(true));
}
