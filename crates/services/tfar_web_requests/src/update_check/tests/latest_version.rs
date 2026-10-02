//! Unit tests for fetching the published version and the background check, against the loopback
//! responder only.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use super::*;
use crate::local_responder::{LocalResponder, ScriptedReply};

const TEST_TIMEOUT: Duration = Duration::from_secs(2);

fn direct_client(timeout: Duration) -> HttpClient {
    HttpClient::new(timeout, ProxyChoice::Direct)
}

/// Runs the background check against one scripted reply and counts the callbacks.
fn run_check(current: &str, reply: ScriptedReply) -> usize {
    let responder = LocalResponder::start(vec![reply]);
    let calls = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&calls);
    let thread = spawn_update_check_with(
        direct_client(TEST_TIMEOUT),
        responder.url("/current_version.txt"),
        current,
        move || {
            counted.fetch_add(1, Ordering::SeqCst);
        },
    )
    .expect("the check thread starts");
    thread.join().expect("the check thread ends");
    calls.load(Ordering::SeqCst)
}

#[test]
fn the_fetch_returns_the_trimmed_body() {
    let responder = LocalResponder::start(vec![ScriptedReply::Body("1.0.0.2\r\n")]);
    let latest = fetch_latest_version_with(
        &direct_client(TEST_TIMEOUT),
        &responder.url("/current_version.txt"),
    )
    .expect("the fetch succeeds");
    assert_eq!(latest, "1.0.0.2");
    assert_eq!(
        responder.next_request_line(),
        "GET /current_version.txt HTTP/1.1"
    );
}

#[test]
fn the_public_fetch_reads_from_the_given_url() {
    let responder = LocalResponder::start(vec![ScriptedReply::Body("0.9.12")]);
    // The public entry point honours a proxy from the environment; the test environment sets
    // none, and NO_PROXY conventions exempt loopback anyway.
    let latest = fetch_latest_version(&responder.url("/current_version.txt"), TEST_TIMEOUT)
        .expect("the fetch succeeds");
    assert_eq!(latest, "0.9.12");
}

#[test]
fn a_newer_body_calls_back_once() {
    assert_eq!(run_check("1.0.0.1", ScriptedReply::Body("1.0.0.2")), 1);
}

#[test]
fn the_same_body_calls_nothing() {
    assert_eq!(run_check("1.0.0.1", ScriptedReply::Body("1.0.0.1")), 0);
}

#[test]
fn an_older_body_calls_nothing() {
    assert_eq!(run_check("0.9.12", ScriptedReply::Body("0.9.11")), 0);
}

#[test]
fn a_garbage_body_calls_nothing() {
    assert_eq!(
        run_check(
            "0.9.12",
            ScriptedReply::Body("<html><body>rate limited</body></html>")
        ),
        0
    );
}

#[test]
fn an_empty_body_calls_nothing() {
    assert_eq!(run_check("0.9.12", ScriptedReply::Body("")), 0);
}

#[test]
fn an_error_status_is_a_failed_fetch_and_calls_nothing() {
    let responder = LocalResponder::start(vec![ScriptedReply::NotFound]);
    let result =
        fetch_latest_version_with(&direct_client(TEST_TIMEOUT), &responder.url("/missing.txt"));
    assert!(matches!(result, Err(Error::Request { .. })), "{result:?}");
    assert_eq!(run_check("0.9.12", ScriptedReply::NotFound), 0);
}

#[test]
fn a_redirect_is_followed_to_the_version_file() {
    let responder = LocalResponder::start(vec![
        ScriptedReply::RedirectTo("/moved/current_version.txt"),
        ScriptedReply::Body("1.0.0.9"),
    ]);
    let latest = fetch_latest_version_with(
        &direct_client(TEST_TIMEOUT),
        &responder.url("/current_version.txt"),
    )
    .expect("the redirect is followed");
    assert_eq!(latest, "1.0.0.9");
    assert_eq!(
        responder.next_request_line(),
        "GET /current_version.txt HTTP/1.1"
    );
    assert_eq!(
        responder.next_request_line(),
        "GET /moved/current_version.txt HTTP/1.1"
    );
}

#[test]
fn a_silent_server_times_out_within_the_timeout() {
    let responder = LocalResponder::start(vec![ScriptedReply::Silence(Duration::from_secs(5))]);
    let timeout = Duration::from_millis(300);
    let started = Instant::now();
    let result = fetch_latest_version_with(&direct_client(timeout), &responder.url("/slow.txt"));
    let elapsed = started.elapsed();
    assert!(
        matches!(result, Err(Error::TimedOut { timeout: reported, .. }) if reported == timeout),
        "{result:?}"
    );
    assert!(elapsed < Duration::from_secs(2), "took {elapsed:?}");
}

#[test]
fn a_timed_out_check_calls_nothing() {
    let responder = LocalResponder::start(vec![ScriptedReply::Silence(Duration::from_secs(5))]);
    let calls = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&calls);
    let started = Instant::now();
    let thread = spawn_update_check_with(
        direct_client(Duration::from_millis(300)),
        responder.url("/current_version.txt"),
        "0.9.12",
        move || {
            counted.fetch_add(1, Ordering::SeqCst);
        },
    )
    .expect("the check thread starts");
    thread.join().expect("the check thread ends");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn the_check_targets_raw_githubusercontent_over_https_with_a_ten_second_timeout() {
    assert!(UPDATE_CHECK_URL.starts_with("https://raw.githubusercontent.com/"));
    assert!(UPDATE_CHECK_URL.ends_with("/master/current_version.txt"));
    assert_eq!(UPDATE_CHECK_TIMEOUT, Duration::from_secs(10));
}

#[test]
fn the_update_message_is_the_originals_text() {
    assert_eq!(
        UPDATE_AVAILABLE_MESSAGE,
        "New version of Task Force Arrowhead Radio is available. Check radio.task-force.ru/en"
    );
}
