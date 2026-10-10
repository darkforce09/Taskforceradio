//! Unit tests for the shared core's audio-side locking.

use std::sync::mpsc;
use std::time::Duration;

use crate::fake_team_speak::{FakeTeamSpeak, SERVER, core_over};

#[test]
fn a_free_state_is_taken_for_audio_without_counting_a_wait() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    core.add_connection(SERVER);
    let Some(connection) = core.connection(SERVER) else {
        panic!("the connection was not added");
    };
    drop(core.lock_state_for_audio(&connection));
    assert!(core.audio_diagnostics().take().is_zero());
}

#[test]
fn an_audio_callback_waits_for_a_held_state_and_counts_the_wait() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let core = core_over(&fake);
    core.add_connection(SERVER);
    let Some(connection) = core.connection(SERVER) else {
        panic!("the connection was not added");
    };
    let hold = Duration::from_millis(50);
    let (locked, is_locked) = mpsc::channel();
    std::thread::scope(|scope| {
        let holder = &connection;
        scope.spawn(move || {
            let guard = holder.state.lock();
            let _ = locked.send(());
            std::thread::sleep(hold);
            drop(guard);
        });
        let _ = is_locked.recv();
        let started = std::time::Instant::now();
        drop(core.lock_state_for_audio(&connection));
        assert!(started.elapsed() >= hold / 2);
    });
    let counts = core.audio_diagnostics().take();
    assert_eq!(counts.contended_state_locks, 1);
    assert_eq!(counts.skipped_blocks, 0);
}
