//! Unit tests for the local push-to-talk state machine.

use std::time::{Duration, Instant};

use super::*;

fn release_at(deadline: Instant) -> PendingRelease {
    PendingRelease {
        server: ServerConnectionHandlerId(1),
        command: "TANGENT\tRELEASED\t1\t1\tdigital\tMe".to_owned(),
        deadline,
    }
}

#[test]
fn a_press_without_a_waiting_release_activates() {
    assert_eq!(PushToTalkState::new().press(), PressAction::Activate);
}

#[test]
fn a_press_while_a_release_waits_cancels_it_for_good() {
    let now = Instant::now();
    let mut state = PushToTalkState::new();
    state.schedule_release(release_at(now));
    assert_eq!(state.press(), PressAction::CancelledPendingRelease);
    assert_eq!(state.take_due_release(now + Duration::from_secs(10)), None);
    assert_eq!(state.next_deadline(), None);
}

#[test]
fn press_release_press_release_within_the_delay_sends_only_the_last_release() {
    // The original's flags let the first release thread consume the second press's skip and
    // send the second release while the third press is held.
    let now = Instant::now();
    let delay = Duration::from_millis(300);
    let mut state = PushToTalkState::new();
    assert_eq!(state.press(), PressAction::Activate);
    state.schedule_release(release_at(now + delay));
    assert_eq!(state.press(), PressAction::CancelledPendingRelease);
    state.schedule_release(release_at(now + delay * 2));
    assert_eq!(state.press(), PressAction::CancelledPendingRelease);
    assert_eq!(state.take_due_release(now + delay * 3), None);
    state.schedule_release(release_at(now + delay * 4));
    assert_eq!(
        state.take_due_release(now + delay * 4),
        Some(release_at(now + delay * 4))
    );
}

#[test]
fn a_release_is_due_only_once_its_deadline_has_passed() {
    let now = Instant::now();
    let mut state = PushToTalkState::new();
    state.schedule_release(release_at(now + Duration::from_millis(100)));
    assert_eq!(state.take_due_release(now), None);
    assert!(
        state
            .take_due_release(now + Duration::from_millis(100))
            .is_some()
    );
    assert_eq!(state.take_due_release(now + Duration::from_secs(1)), None);
}

#[test]
fn the_voice_activation_state_is_kept_for_the_release() {
    let mut state = PushToTalkState::new();
    assert!(!state.was_voice_activation_on());
    state.save_voice_activation(true);
    assert!(state.was_voice_activation_on());
}

#[test]
fn a_stopping_state_gives_up_its_waiting_release() {
    let now = Instant::now();
    let mut state = PushToTalkState::new();
    state.schedule_release(release_at(now + Duration::from_secs(60)));
    state.request_stop();
    assert!(state.is_stopping());
    assert!(state.take_any_release().is_some());
}
