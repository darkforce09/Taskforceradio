//! Unit tests for the remote tangent apply function.

use std::cell::Cell;
use std::sync::Arc;
use std::time::Duration;

use super::*;
use crate::test_support::{ALPHA, BRAVO, at, heard_on, now, session, tangent};

/// A route that hears every sender on one carried short-range radio.
fn heard_on_radio(_: &SessionState, _: &ClientData, _: IgnoreTangents) -> Vec<HeardVia> {
    vec![heard_on(ReceivedOn::ShortRange)]
}

#[test]
fn a_heard_press_plays_the_start_sound_and_counts_the_reception() {
    let mut session = session();
    let effects = session
        .apply_remote_tangent(&tangent("alpha", true, "100"), now(), heard_on_radio)
        .unwrap();
    assert_eq!(effects.transition, TangentTransition::Pressed);
    let start = effects.play_start.unwrap();
    assert_eq!(start.sound, "radio-sounds/sw/remote_start");
    let placement = start.placements.first().unwrap();
    assert!((placement.gain - 0.4096).abs() < 1e-12);
    assert!(placement.is_same_vehicle && !placement.is_on_ground);
    assert!(effects.play_end.is_none());
    assert!(effects.is_receiving_changed);
    assert_eq!(effects.heard.len(), 1);
    let alpha = session.clients.get("alpha").unwrap();
    assert_eq!(alpha.transmitting_over, Some(TangentRadio::ShortRange));
    assert_eq!(alpha.transmitting_frequency.as_deref(), Some("100"));
    assert_eq!(alpha.range, 3000);
    assert_eq!(
        session
            .clients
            .own()
            .unwrap()
            .receiving_transmission_count(),
        1
    );
}

/// F-015: the original sets `playPressed` only on a flip, so a repeated PRESSED plays the
/// remote end sound and counts the reception out; here it changes no sound and no count.
#[test]
fn a_repeated_pressed_plays_no_sound_and_changes_no_count() {
    let mut session = session();
    session.apply_remote_tangent(&tangent("alpha", true, "100"), now(), heard_on_radio);
    let effects = session
        .apply_remote_tangent(&tangent("alpha", true, "100"), now(), heard_on_radio)
        .unwrap();
    assert_eq!(effects.transition, TangentTransition::RepeatedPress);
    assert!(effects.play_start.is_none());
    assert!(effects.play_end.is_none());
    assert!(!effects.is_receiving_changed);
    assert_eq!(
        session
            .clients
            .own()
            .unwrap()
            .receiving_transmission_count(),
        1
    );
}

#[test]
fn a_release_is_routed_before_the_reset_with_the_senders_abilities() {
    let mut session = session();
    session.apply_remote_tangent(&tangent("alpha", true, "100"), now(), heard_on_radio);
    session.clients.get_mut("alpha").unwrap().can_use_lr_radio = true;
    let seen = Cell::new(None);
    let effects = session
        .apply_remote_tangent(
            &tangent("alpha", false, "100"),
            now(),
            |_, sender, ignore| {
                seen.set(Some((sender.range, sender.is_transmitting(), ignore)));
                vec![heard_on(ReceivedOn::ShortRange)]
            },
        )
        .unwrap();
    assert_eq!(
        seen.get(),
        Some((
            3000,
            true,
            IgnoreTangents {
                short_range: true,
                long_range: true
            }
        ))
    );
    assert_eq!(effects.transition, TangentTransition::Released);
    assert_eq!(
        effects.play_end.unwrap().sound,
        "radio-sounds/sw/remote_end"
    );
    assert!(effects.is_receiving_changed);
    let alpha = session.clients.get("alpha").unwrap();
    assert!(!alpha.is_transmitting());
    assert_eq!(alpha.range, 0);
    assert_eq!(alpha.transmitting_frequency, None);
}

#[test]
fn a_press_is_routed_after_the_update() {
    let mut session = session();
    let seen = Cell::new(None);
    session.apply_remote_tangent(&tangent("alpha", true, "100"), now(), |_, sender, _| {
        seen.set(Some((sender.range, sender.is_transmitting())));
        Vec::new()
    });
    assert_eq!(seen.get(), Some((3000, true)));
}

/// A further original bug, fixed: the original counts with a number and a set of frequencies, so two senders
/// on one frequency count 2 but the first release removes the frequency and the second finds
/// nothing, leaving the player receiving forever. Here each sender counts and leaves alone.
#[test]
fn two_senders_on_one_frequency_each_count_and_each_release_ends_its_own() {
    let mut session = session();
    session.apply_remote_tangent(&tangent("alpha", true, "100"), now(), heard_on_radio);
    session.apply_remote_tangent(&tangent("bravo", true, "100"), now(), heard_on_radio);
    let own = session.clients.own().unwrap();
    assert_eq!(own.receiving_transmission_count(), 2);
    session.apply_remote_tangent(&tangent("alpha", false, "100"), now(), heard_on_radio);
    let own = session.clients.own().unwrap();
    assert_eq!(own.receiving_senders().collect::<Vec<_>>(), [BRAVO]);
    session.apply_remote_tangent(&tangent("bravo", false, "100"), now(), heard_on_radio);
    assert!(!session.clients.own().unwrap().is_receiving_transmission());
}

#[test]
fn a_dead_player_hears_no_sound_and_counts_nothing() {
    let mut session = session();
    session.game.is_alive = false;
    let effects = session
        .apply_remote_tangent(&tangent("alpha", true, "100"), now(), heard_on_radio)
        .unwrap();
    assert!(effects.play_start.is_none());
    assert!(!effects.is_receiving_changed);
    assert_eq!(effects.heard.len(), 1);
}

#[test]
fn the_intercom_plays_no_radio_sound() {
    let mut session = session();
    let effects = session
        .apply_remote_tangent(&tangent("alpha", true, "100"), now(), |_, _, _| {
            vec![heard_on(ReceivedOn::Intercom)]
        })
        .unwrap();
    assert!(effects.play_start.is_none());
    assert!(!effects.is_receiving_changed);
}

#[test]
fn a_speaker_sound_carries_its_place_vehicle_loss_and_water() {
    let mut session = session();
    Arc::make_mut(&mut session.clients.own_mut().unwrap().vehicle).isolation = 0.3;
    let effects = session
        .apply_remote_tangent(&tangent("alpha", true, "100"), now(), |_, _, _| {
            let mut via = heard_on(ReceivedOn::Ground);
            via.position = Some(at(1.0, 2.0, -3.0));
            via.wave_z = -2.0;
            let vehicle = Arc::make_mut(&mut via.vehicle);
            vehicle.isolation = 0.9;
            vehicle.name = "2:5".to_owned();
            vec![via]
        })
        .unwrap();
    let placement = effects.play_start.unwrap().placements.pop().unwrap();
    assert!(placement.is_on_ground && placement.is_underwater && !placement.is_same_vehicle);
    assert!((placement.vehicle_volume_loss - 0.99).abs() < 1e-12);
    assert_eq!(placement.position, Some(at(1.0, 2.0, -3.0)));
}

/// A further original bug, fixed: the original stamps a tangent into the position time, which makes the
/// extrapolated position of a moving sender jump back; a tangent here only counts as activity.
#[test]
fn a_tangent_press_keeps_the_position_extrapolation_clock() {
    let start = now();
    let mut session = session();
    let alpha = session.clients.get_mut("alpha").unwrap();
    alpha.position_updated_at = Some(start);
    Arc::make_mut(&mut alpha.vehicle).velocity = [10.0, 0.0, 0.0];
    let later = start + Duration::from_secs(1);
    session.apply_remote_tangent(&tangent("alpha", true, "100"), later, heard_on_radio);
    let alpha = session.clients.get("alpha").unwrap();
    assert_eq!(alpha.position_updated_at, Some(start));
    assert_eq!(alpha.last_activity_at, Some(later));
    assert_eq!(alpha.client_position(later), at(10.0, 0.0, 0.0));
}

#[test]
fn a_long_range_press_marks_the_sender_long_range_capable() {
    let mut session = session();
    let mut press = tangent("alpha", true, "40");
    press.kind = TangentKind::LongRange;
    press.subtype = tfar_arma_values::RadioSubtype::DigitalLongRange;
    let effects = session
        .apply_remote_tangent(&press, now(), heard_on_radio)
        .unwrap();
    assert_eq!(
        effects.play_start.unwrap().sound,
        "radio-sounds/lr/remote_start"
    );
    let alpha = session.clients.get("alpha").unwrap();
    assert!(alpha.can_use_lr_radio);
    assert_eq!(alpha.transmitting_over, Some(TangentRadio::LongRange));
}

#[test]
fn own_and_unknown_broadcasts_are_ignored() {
    let mut session = session();
    assert!(
        session
            .apply_remote_tangent(&tangent("me", true, "100"), now(), heard_on_radio)
            .is_none()
    );
    assert!(
        session
            .apply_remote_tangent(&tangent("ghost", true, "100"), now(), heard_on_radio)
            .is_none()
    );
    assert_eq!(session.clients.get("alpha").unwrap().client_id(), ALPHA);
}
