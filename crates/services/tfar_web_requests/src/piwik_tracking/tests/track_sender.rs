//! Unit tests for the fire-and-forget sender, against the loopback responder only.

use std::time::{Duration, Instant};

use super::*;
use crate::local_responder::{LocalResponder, ScriptedReply};

#[test]
fn the_request_reaches_the_server_with_its_path_and_query() {
    let responder = LocalResponder::start(vec![ScriptedReply::Body("GIF89a")]);
    let url = Url::parse(&responder.url("/piwik.php?idsite=2&rec=1&cvar={%221%22:[%22a%22]}"))
        .expect("a loopback URL");
    let thread = spawn_track_with(HttpClient::new(TRACK_TIMEOUT, ProxyChoice::Direct), url)
        .expect("the tracking thread starts");
    thread.join().expect("the tracking thread ends");
    assert_eq!(
        responder.next_request_line(),
        "GET /piwik.php?idsite=2&rec=1&cvar={%221%22:[%22a%22]} HTTP/1.1"
    );
}

#[test]
fn a_dead_host_fails_quietly_within_the_timeout() {
    let responder = LocalResponder::start(vec![ScriptedReply::Silence(Duration::from_secs(5))]);
    let url = Url::parse(&responder.url("/piwik.php")).expect("a loopback URL");
    let started = Instant::now();
    let thread = spawn_track_with(
        HttpClient::new(Duration::from_millis(300), ProxyChoice::Direct),
        url,
    )
    .expect("the tracking thread starts");
    thread
        .join()
        .expect("the tracking thread ends without panicking");
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn an_error_status_fails_quietly() {
    let responder = LocalResponder::start(vec![ScriptedReply::NotFound]);
    let url = Url::parse(&responder.url("/piwik.php")).expect("a loopback URL");
    let thread = spawn_track_with(HttpClient::new(TRACK_TIMEOUT, ProxyChoice::Direct), url)
        .expect("the tracking thread starts");
    thread
        .join()
        .expect("the tracking thread ends without panicking");
}

#[test]
fn the_tracking_timeout_is_five_seconds() {
    assert_eq!(TRACK_TIMEOUT, Duration::from_secs(5));
}
