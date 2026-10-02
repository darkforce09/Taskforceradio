//! Unit tests for the handle's state machine, run headless through a fake `WindowBackend`.

use std::sync::atomic::AtomicUsize;
use std::thread::ThreadId;
use std::time::Instant;

use parking_lot::Mutex;

use super::*;
use crate::debug_snapshot::DebugSection;
use crate::window_control::Waker;

/// How the fake window behaves once opened.
#[derive(Clone, Copy, Debug)]
enum FakeBehaviour {
    /// Stays open until the session says to close, sleeping between wakes.
    StaysOpenUntilAsked,
    /// Draws one frame, then the "user" closes it.
    UserClosesAfterOneFrame,
    /// Fails to open.
    FailsToOpen,
    /// Ignores close requests and stays open this long.
    IgnoresClose(Duration),
}

/// What the fake saw, shared with the test.
#[derive(Default)]
struct FakeLog {
    titles: Vec<String>,
    threads: Vec<ThreadId>,
    snapshots_seen: Vec<Arc<DebugSnapshot>>,
    is_dropped: bool,
}

struct FakeBackend {
    behaviour: FakeBehaviour,
    log: Arc<Mutex<FakeLog>>,
    wakes: Arc<AtomicUsize>,
}

impl WindowBackend for FakeBackend {
    fn run_window(&mut self, session: &WindowSession<'_>) -> Result<()> {
        {
            let mut log = self.log.lock();
            log.titles.push(session.title().to_owned());
            log.threads.push(std::thread::current().id());
        }
        match self.behaviour {
            FakeBehaviour::FailsToOpen => Err(Error::Window {
                reason: "no OpenGL context".to_owned(),
            }),
            FakeBehaviour::UserClosesAfterOneFrame => {
                self.log.lock().snapshots_seen.push(session.snapshot());
                Ok(())
            }
            FakeBehaviour::IgnoresClose(duration) => {
                std::thread::sleep(duration);
                Ok(())
            }
            FakeBehaviour::StaysOpenUntilAsked => {
                let ui_thread = std::thread::current();
                let wakes = Arc::clone(&self.wakes);
                session.set_waker(Waker::new(move || {
                    wakes.fetch_add(1, Ordering::SeqCst);
                    ui_thread.unpark();
                }));
                loop {
                    let snapshot = session.snapshot();
                    {
                        let mut log = self.log.lock();
                        let is_new = log
                            .snapshots_seen
                            .last()
                            .is_none_or(|last| !Arc::ptr_eq(last, &snapshot));
                        if is_new {
                            log.snapshots_seen.push(snapshot);
                        }
                    }
                    if session.should_close() {
                        return Ok(());
                    }
                    // Long enough that anything a test sees within its two-second wait came
                    // through the waker, not a timeout.
                    std::thread::park_timeout(Duration::from_secs(5));
                }
            }
        }
    }
}

impl Drop for FakeBackend {
    fn drop(&mut self) {
        self.log.lock().is_dropped = true;
    }
}

struct Fixture {
    handle: DebugWindowHandle,
    log: Arc<Mutex<FakeLog>>,
    wakes: Arc<AtomicUsize>,
}

fn spawn_fake(behaviour: FakeBehaviour) -> Fixture {
    let log = Arc::new(Mutex::new(FakeLog::default()));
    let wakes = Arc::new(AtomicUsize::new(0));
    let backend = FakeBackend {
        behaviour,
        log: Arc::clone(&log),
        wakes: Arc::clone(&wakes),
    };
    let handle = spawn_with_backend("TFAR Debug UI", backend).expect("the UI thread starts");
    Fixture { handle, log, wakes }
}

/// Polls `condition` every 5 ms for up to two seconds.
fn wait_until(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    condition()
}

fn opened_count(fixture: &Fixture) -> usize {
    fixture.log.lock().titles.len()
}

fn sample_snapshot(title: &str) -> DebugSnapshot {
    DebugSnapshot::new().with_section(DebugSection::key_values(title, [("session", "up")]))
}

#[test]
fn spawn_starts_hidden_with_no_window_open() {
    let fixture = spawn_fake(FakeBehaviour::StaysOpenUntilAsked);
    std::thread::sleep(Duration::from_millis(50));
    assert!(!fixture.handle.is_open());
    assert_eq!(opened_count(&fixture), 0);
}

#[test]
fn show_opens_one_window_with_the_title() {
    let fixture = spawn_fake(FakeBehaviour::StaysOpenUntilAsked);
    fixture.handle.show();
    assert!(wait_until(|| opened_count(&fixture) == 1));
    assert!(fixture.handle.is_open());
    assert_eq!(fixture.log.lock().titles, ["TFAR Debug UI"]);
}

#[test]
fn show_while_shown_does_not_open_a_second_window() {
    let fixture = spawn_fake(FakeBehaviour::StaysOpenUntilAsked);
    fixture.handle.show();
    assert!(wait_until(|| opened_count(&fixture) == 1));
    fixture.handle.show();
    fixture.handle.show();
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(opened_count(&fixture), 1);
    assert!(fixture.handle.is_open());
}

#[test]
fn hide_closes_the_window_and_show_reopens_it_on_the_same_thread() {
    let fixture = spawn_fake(FakeBehaviour::StaysOpenUntilAsked);
    fixture.handle.show();
    assert!(wait_until(|| fixture.handle.is_open()));
    fixture.handle.hide();
    assert!(wait_until(|| !fixture.handle.is_open()));
    fixture.handle.show();
    assert!(wait_until(|| opened_count(&fixture) == 2));
    assert!(fixture.handle.is_open());
    let log = fixture.log.lock();
    assert_eq!(log.threads.len(), 2);
    assert_eq!(log.threads[0], log.threads[1]);
}

#[test]
fn a_snapshot_published_while_open_reaches_the_window_at_once() {
    let fixture = spawn_fake(FakeBehaviour::StaysOpenUntilAsked);
    fixture.handle.show();
    assert!(wait_until(|| fixture.log.lock().snapshots_seen.len() == 1));
    let wakes_before = fixture.wakes.load(Ordering::SeqCst);
    fixture.handle.publish(sample_snapshot("Clients"));
    assert!(wait_until(|| fixture.log.lock().snapshots_seen.len() == 2));
    assert!(fixture.wakes.load(Ordering::SeqCst) > wakes_before);
    let log = fixture.log.lock();
    assert!(log.snapshots_seen[0].is_empty());
    assert_eq!(*log.snapshots_seen[1], sample_snapshot("Clients"));
}

#[test]
fn a_snapshot_published_while_hidden_is_shown_on_the_next_open() {
    let fixture = spawn_fake(FakeBehaviour::UserClosesAfterOneFrame);
    fixture.handle.publish(sample_snapshot("Game link"));
    fixture.handle.show();
    assert!(wait_until(|| fixture.log.lock().snapshots_seen.len() == 1));
    assert_eq!(
        *fixture.log.lock().snapshots_seen[0],
        sample_snapshot("Game link")
    );
}

#[test]
fn a_window_the_user_closes_stays_closed_until_the_next_show() {
    let fixture = spawn_fake(FakeBehaviour::UserClosesAfterOneFrame);
    fixture.handle.show();
    assert!(wait_until(
        || opened_count(&fixture) == 1 && !fixture.handle.is_open()
    ));
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(opened_count(&fixture), 1);
    fixture.handle.show();
    assert!(wait_until(|| opened_count(&fixture) == 2));
}

#[test]
fn a_window_that_fails_to_open_reports_why_once() {
    let fixture = spawn_fake(FakeBehaviour::FailsToOpen);
    assert_eq!(fixture.handle.take_window_error(), None);
    fixture.handle.show();
    assert!(wait_until(
        || opened_count(&fixture) == 1 && !fixture.handle.is_open()
    ));
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(
        opened_count(&fixture),
        1,
        "a failed window is not retried on its own"
    );
    assert_eq!(
        fixture.handle.take_window_error().as_deref(),
        Some("the debug window could not open: no OpenGL context")
    );
    assert_eq!(fixture.handle.take_window_error(), None);
}

#[test]
fn repeated_show_and_hide_settles_on_the_last_request() {
    let fixture = spawn_fake(FakeBehaviour::StaysOpenUntilAsked);
    for _ in 0..20 {
        fixture.handle.show();
        fixture.handle.hide();
    }
    fixture.handle.show();
    assert!(wait_until(|| fixture.handle.is_open()));
    for _ in 0..20 {
        fixture.handle.hide();
        fixture.handle.show();
    }
    fixture.handle.hide();
    assert!(wait_until(|| !fixture.handle.is_open()));
    std::thread::sleep(Duration::from_millis(50));
    assert!(!fixture.handle.is_open());
}

#[test]
fn shutdown_while_open_closes_the_window_and_joins_within_a_second() {
    let fixture = spawn_fake(FakeBehaviour::StaysOpenUntilAsked);
    fixture.handle.show();
    assert!(wait_until(|| fixture.handle.is_open()));
    let started = Instant::now();
    fixture.handle.shutdown().expect("the UI thread ends");
    assert!(started.elapsed() < SHUTDOWN_TIMEOUT);
    assert!(fixture.log.lock().is_dropped);
}

#[test]
fn shutdown_while_hidden_joins_within_a_second() {
    let fixture = spawn_fake(FakeBehaviour::StaysOpenUntilAsked);
    let started = Instant::now();
    fixture.handle.shutdown().expect("the UI thread ends");
    assert!(started.elapsed() < SHUTDOWN_TIMEOUT);
    assert!(fixture.log.lock().is_dropped);
    assert_eq!(fixture.log.lock().titles.len(), 0);
}

#[test]
fn a_window_that_never_closes_makes_shutdown_time_out_after_one_second() {
    let fixture = spawn_fake(FakeBehaviour::IgnoresClose(Duration::from_secs(3)));
    fixture.handle.show();
    assert!(wait_until(|| opened_count(&fixture) == 1));
    let started = Instant::now();
    let result = fixture.handle.shutdown();
    let elapsed = started.elapsed();
    assert!(
        matches!(result, Err(Error::ShutdownTimedOut { waited }) if waited == SHUTDOWN_TIMEOUT),
        "{result:?}"
    );
    assert!(elapsed >= SHUTDOWN_TIMEOUT, "returned after {elapsed:?}");
    assert!(
        elapsed < SHUTDOWN_TIMEOUT + Duration::from_millis(500),
        "took {elapsed:?}"
    );
}

#[test]
fn dropping_the_handle_shuts_the_thread_down() {
    let fixture = spawn_fake(FakeBehaviour::StaysOpenUntilAsked);
    fixture.handle.show();
    assert!(wait_until(|| fixture.handle.is_open()));
    let Fixture { handle, log, .. } = fixture;
    drop(handle);
    assert!(log.lock().is_dropped);
}

#[test]
fn the_display_check_refuses_an_unset_or_empty_display() {
    assert!(matches!(require_display(None), Err(Error::NoDisplay)));
    assert!(matches!(
        require_display(Some(OsStr::new(""))),
        Err(Error::NoDisplay)
    ));
    assert!(require_display(Some(OsStr::new(":0"))).is_ok());
    assert!(require_display(Some(OsStr::new(":99.0"))).is_ok());
}

#[test]
fn a_second_claim_of_the_process_slot_is_refused() {
    let flag = AtomicBool::new(false);
    assert!(claim_process_slot(&flag).is_ok());
    assert!(matches!(
        claim_process_slot(&flag),
        Err(Error::AlreadySpawned)
    ));
    assert!(matches!(
        claim_process_slot(&flag),
        Err(Error::AlreadySpawned)
    ));
}
