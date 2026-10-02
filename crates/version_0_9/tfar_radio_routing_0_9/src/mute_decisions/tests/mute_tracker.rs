//! Unit tests for `MuteTracker`: requests only on a change, unmutes only of what this plugin
//! muted.

use super::*;

#[test]
fn a_mute_is_requested_once_and_an_unmute_only_after_it() {
    let mut tracker = MuteTracker::new();
    assert_eq!(tracker.decide(ClientId(5), true).mute, [ClientId(5)]);
    assert!(
        tracker.decide(ClientId(5), true).is_empty(),
        "already muted"
    );
    assert!(tracker.is_muted(ClientId(5)));
    assert_eq!(tracker.decide(ClientId(5), false).unmute, [ClientId(5)]);
    assert!(
        tracker.decide(ClientId(5), false).is_empty(),
        "already unmuted"
    );
}

#[test]
fn an_unmute_never_reaches_a_client_this_plugin_did_not_mute() {
    let mut tracker = MuteTracker::new();
    assert!(tracker.decide(ClientId(5), false).is_empty());
}

#[test]
fn client_id_zero_is_never_requested() {
    let mut tracker = MuteTracker::new();
    assert!(tracker.decide(ClientId(0), true).is_empty());
    assert!(!tracker.is_muted(ClientId(0)));
}

#[test]
fn unmute_all_unmutes_exactly_the_muted_clients() {
    let mut tracker = MuteTracker::new();
    tracker.decide(ClientId(7), true);
    tracker.decide(ClientId(3), true);
    assert_eq!(
        tracker.unmute_all(),
        MuteDecision {
            mute: Vec::new(),
            unmute: vec![ClientId(3), ClientId(7)],
        }
    );
    assert_eq!(tracker.muted().count(), 0);
    assert!(tracker.unmute_all().is_empty());
}

#[test]
fn a_forgotten_or_departed_client_is_dropped_without_a_request() {
    let mut tracker = MuteTracker::new();
    tracker.decide(ClientId(1), true);
    tracker.decide(ClientId(2), true);
    tracker.decide(ClientId(3), true);
    tracker.forget(ClientId(1));
    tracker.retain_present(&[ClientId(3), ClientId(9)]);
    assert_eq!(tracker.muted().collect::<Vec<_>>(), [ClientId(3)]);
}

#[test]
fn merged_decisions_keep_every_request_in_order() {
    let mut decision = MuteDecision {
        mute: vec![ClientId(1)],
        unmute: vec![ClientId(2)],
    };
    decision.merge(MuteDecision {
        mute: vec![ClientId(3)],
        unmute: Vec::new(),
    });
    assert_eq!(decision.mute, [ClientId(1), ClientId(3)]);
    assert_eq!(decision.unmute, [ClientId(2)]);
}
