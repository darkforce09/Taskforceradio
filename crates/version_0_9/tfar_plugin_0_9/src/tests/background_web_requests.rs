//! Unit tests for the plugin's running web requests.
//!
//! A request needs a network, so each test that starts one re-runs this test binary as a child
//! whose `ALL_PROXY` points at 127.0.0.1: at a port nothing listens on (the request fails at
//! once) or at a listener of the parent that never answers (the request hangs until its
//! timeout). No request leaves the machine.

use std::net::TcpListener;
use std::process::Command;
use std::time::{Duration, Instant};

use tfar_web_requests::{build_track_request_0_9, spawn_cancellable_track};

use super::*;

/// Set in the child's environment to the name of the test it runs.
const CHILD_MARKER: &str = "TFAR_PLUGIN_0_9_WEB_REQUEST_CHILD";

/// Runs `test` in a child of this test binary whose web requests go through `proxy_port`;
/// in the child itself, runs it here.
fn run_in_child(test_name: &str, proxy_port: u16, test: impl FnOnce()) {
    if std::env::var(CHILD_MARKER).as_deref() == Ok(test_name) {
        test();
        return;
    }
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([test_name, "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD_MARKER, test_name);
    for variable in [
        "ALL_PROXY",
        "all_proxy",
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
        "NO_PROXY",
        "no_proxy",
    ] {
        command.env_remove(variable);
    }
    let status = command
        .env("ALL_PROXY", format!("http://127.0.0.1:{proxy_port}"))
        .status()
        .unwrap();
    assert!(status.success(), "the test failed in the child: {status}");
}

/// A port nothing listens on: bound once, then released, so a connection is refused.
fn refusing_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// A tracking request through the child's proxy.
fn tracking_request() -> BackgroundRequest {
    spawn_cancellable_track(build_track_request_0_9("unit-test").unwrap()).unwrap()
}

/// Waits up to 5 s for `request` to end.
fn wait_until_finished(request: &BackgroundRequest) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !request.is_finished() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(request.is_finished());
}

#[test]
fn an_empty_set_joins_at_once() {
    let requests = BackgroundWebRequests::new();
    let started = Instant::now();
    assert_eq!(
        requests.cancel_and_join(started + Duration::from_secs(1)),
        0
    );
    assert!(started.elapsed() < Duration::from_millis(100));
    assert!(requests.is_empty());
}

#[test]
fn a_finished_request_is_joined_when_the_next_one_is_kept() {
    run_in_child(
        "background_web_requests::tests::a_finished_request_is_joined_when_the_next_one_is_kept",
        refusing_port(),
        || {
            let requests = BackgroundWebRequests::new();
            let first = tracking_request();
            wait_until_finished(&first);
            requests.keep(first);
            requests.keep(tracking_request());
            assert_eq!(requests.len(), 1);
            assert_eq!(
                requests.cancel_and_join(Instant::now() + Duration::from_secs(2)),
                0
            );
        },
    );
}

#[test]
fn a_request_that_ends_is_joined_at_shutdown() {
    run_in_child(
        "background_web_requests::tests::a_request_that_ends_is_joined_at_shutdown",
        refusing_port(),
        || {
            let requests = BackgroundWebRequests::new();
            requests.keep(tracking_request());
            requests.keep(tracking_request());
            assert_eq!(
                requests.cancel_and_join(Instant::now() + Duration::from_secs(2)),
                0
            );
            assert!(requests.is_empty());
        },
    );
}

#[test]
fn a_request_in_flight_never_holds_the_shutdown_past_its_deadline() {
    // The parent's listener takes the connection into its backlog and never answers.
    let silent_proxy = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = silent_proxy.local_addr().unwrap().port();
    run_in_child(
        "background_web_requests::tests::a_request_in_flight_never_holds_the_shutdown_past_its_deadline",
        port,
        || {
            let requests = BackgroundWebRequests::new();
            requests.keep(tracking_request());
            std::thread::sleep(Duration::from_millis(200));
            let started = Instant::now();
            assert_eq!(
                requests.cancel_and_join(started + Duration::from_millis(300)),
                1
            );
            let waited = started.elapsed();
            assert!(waited >= Duration::from_millis(300), "{waited:?}");
            assert!(waited < Duration::from_millis(800), "{waited:?}");
        },
    );
    drop(silent_proxy);
}

#[test]
fn a_request_kept_after_the_close_is_cancelled_and_not_kept() {
    run_in_child(
        "background_web_requests::tests::a_request_kept_after_the_close_is_cancelled_and_not_kept",
        refusing_port(),
        || {
            let requests = BackgroundWebRequests::new();
            requests.cancel_all();
            requests.keep(tracking_request());
            assert!(requests.is_empty());
            assert_eq!(requests.cancel_and_join(Instant::now()), 0);
        },
    );
}
