//! Unit tests for the `VOLUME` and `RELEASE_ALL_TANGENTS` apply functions.

use crate::client_state::test_support::{ALPHA, now, session};

#[test]
fn a_volume_sets_range_and_talking_and_marks_the_plugin_enabled() {
    let start = now();
    let mut session = session();
    let applied = session
        .apply_remote_volume("alpha", 40, true, start)
        .unwrap();
    assert_eq!(applied.client_id, ALPHA);
    assert!(!applied.is_own_client);
    let alpha = session.clients.get("alpha").unwrap();
    assert_eq!(alpha.voice_volume, 40);
    assert!(alpha.is_talking_now);
    assert_eq!(alpha.cached_plugin_enabled(start), Some(true));
    assert!(
        session
            .apply_remote_volume("ghost", 1, false, start)
            .is_none()
    );
    assert!(
        session
            .apply_remote_volume("me", 1, false, start)
            .unwrap()
            .is_own_client
    );
}

/// A further original bug, fixed: the original's `RELEASE_ALL_TANGENTS` only clears the tangent, so a
/// transmission the local player received stays counted; here it ends that reception.
#[test]
fn release_all_tangents_stops_the_reception_it_ended() {
    let mut session = session();
    let alpha = session.clients.get_mut("alpha").unwrap();
    alpha.transmitting_over = Some(crate::client_state::TangentRadio::ShortRange);
    alpha.transmitting_frequency = Some("100".to_owned());
    alpha.range = 3000;
    session
        .clients
        .own_mut()
        .unwrap()
        .receiving
        .start(ALPHA, "100");
    let applied = session.apply_release_all_tangents("alpha").unwrap();
    assert!(applied.is_receiving_changed);
    let alpha = session.clients.get("alpha").unwrap();
    assert!(!alpha.is_transmitting());
    assert_eq!(alpha.range, 0);
    assert!(!session.clients.own().unwrap().is_receiving_transmission());
    assert!(
        !session
            .apply_release_all_tangents("alpha")
            .unwrap()
            .is_receiving_changed
    );
    assert!(session.apply_release_all_tangents("ghost").is_none());
}
