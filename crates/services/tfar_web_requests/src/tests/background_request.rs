//! Unit tests for `BackgroundRequest`: the cancel flag the worker sees, the join with a deadline
//! and the drop that cancels without waiting, on plain worker closures (no HTTP).

use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::*;

/// A worker that waits until `release` sends (or disconnects), then reports whether it saw the
/// cancel flag set.
fn held_worker(
    release: mpsc::Receiver<()>,
    saw_cancel: mpsc::Sender<bool>,
) -> impl FnOnce(&CancelFlag) + Send + 'static {
    move |cancel: &CancelFlag| {
        let _ = release.recv();
        let _ = saw_cancel.send(cancel.is_cancelled());
    }
}

#[test]
fn a_finished_worker_joins_at_once() {
    let mut request = BackgroundRequest::spawn("test-quick", CancelFlag::new(), |_| {}).unwrap();
    assert!(request.join_timeout(Duration::from_secs(2)));
    assert!(request.is_finished());
    assert!(
        request.join_timeout(Duration::ZERO),
        "a joined request joins again at once"
    );
    assert!(!request.is_cancelled());
}

#[test]
fn the_worker_sees_a_cancel_made_while_it_runs() {
    let (release_sender, release) = mpsc::channel();
    let (saw_cancel_sender, saw_cancel) = mpsc::channel();
    let mut request = BackgroundRequest::spawn(
        "test-cancel",
        CancelFlag::new(),
        held_worker(release, saw_cancel_sender),
    )
    .unwrap();
    request.cancel();
    assert!(request.is_cancelled());
    release_sender.send(()).unwrap();
    assert_eq!(saw_cancel.recv_timeout(Duration::from_secs(2)), Ok(true));
    assert!(request.join_timeout(Duration::from_secs(2)));
}

#[test]
fn a_flag_cancelled_before_the_spawn_is_set_when_the_worker_starts() {
    let cancel = CancelFlag::new();
    cancel.cancel();
    let (saw_cancel_sender, saw_cancel) = mpsc::channel();
    let mut request = BackgroundRequest::spawn("test-precancelled", cancel, move |cancel| {
        let _ = saw_cancel_sender.send(cancel.is_cancelled());
    })
    .unwrap();
    assert_eq!(saw_cancel.recv_timeout(Duration::from_secs(2)), Ok(true));
    assert!(request.is_cancelled());
    assert!(request.join_timeout(Duration::from_secs(2)));
}

#[test]
fn join_timeout_returns_false_at_the_deadline_while_the_worker_runs() {
    let (release_sender, release) = mpsc::channel();
    let (saw_cancel_sender, _saw_cancel) = mpsc::channel();
    let mut request = BackgroundRequest::spawn(
        "test-slow",
        CancelFlag::new(),
        held_worker(release, saw_cancel_sender),
    )
    .unwrap();
    let started = Instant::now();
    assert!(!request.join_timeout(Duration::from_millis(100)));
    let waited = started.elapsed();
    assert!(waited >= Duration::from_millis(100), "{waited:?}");
    assert!(waited < Duration::from_secs(1), "{waited:?}");
    assert!(!request.is_finished());
    assert!(!request.is_cancelled(), "a join does not cancel");
    release_sender.send(()).unwrap();
    assert!(request.join_timeout(Duration::from_secs(2)));
    assert!(request.is_finished());
}

#[test]
fn dropping_the_handle_cancels_without_waiting_for_the_worker() {
    let (release_sender, release) = mpsc::channel();
    let (saw_cancel_sender, saw_cancel) = mpsc::channel();
    let request = BackgroundRequest::spawn(
        "test-dropped",
        CancelFlag::new(),
        held_worker(release, saw_cancel_sender),
    )
    .unwrap();
    let started = Instant::now();
    drop(request);
    assert!(started.elapsed() < Duration::from_millis(100));
    release_sender.send(()).unwrap();
    assert_eq!(
        saw_cancel.recv_timeout(Duration::from_secs(2)),
        Ok(true),
        "the detached worker sees the cancel"
    );
}
