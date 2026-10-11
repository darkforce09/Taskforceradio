//! Unit tests for `MuteTracker`.

use super::*;

const ALPHA: TeamspeakClientId = TeamspeakClientId(2);
const BRAVO: TeamspeakClientId = TeamspeakClientId(3);

#[test]
fn only_a_change_of_state_reaches_teamspeak() {
    let mut tracker = MuteTracker::new();
    assert!(!tracker.set_muted(ALPHA, false));
    assert!(tracker.set_muted(ALPHA, true));
    assert!(!tracker.set_muted(ALPHA, true));
    assert!(tracker.is_muted(ALPHA));
    assert!(tracker.set_muted(ALPHA, false));
}

#[test]
fn unmute_all_returns_the_tracked_set_and_forgets_it() {
    let mut tracker = MuteTracker::new();
    tracker.set_muted(BRAVO, true);
    tracker.set_muted(ALPHA, true);
    assert_eq!(tracker.unmute_all(), [ALPHA, BRAVO]);
    assert_eq!(tracker.muted().count(), 0);
    assert_eq!(tracker.unmute_all(), []);
}

#[test]
fn applying_changes_keeps_only_the_effective_ones() {
    let mut tracker = MuteTracker::new();
    tracker.set_muted(ALPHA, true);
    let effective = tracker.apply(&MuteChanges {
        mute: vec![ALPHA, BRAVO],
        unmute: vec![TeamspeakClientId(9)],
    });
    assert_eq!(effective.mute, [BRAVO]);
    assert_eq!(effective.unmute, []);
}

#[test]
fn a_leaving_client_is_forgotten() {
    let mut tracker = MuteTracker::new();
    tracker.set_muted(ALPHA, true);
    assert!(tracker.client_left(ALPHA));
    assert!(!tracker.client_left(ALPHA));
    assert!(!tracker.is_muted(ALPHA));
}
