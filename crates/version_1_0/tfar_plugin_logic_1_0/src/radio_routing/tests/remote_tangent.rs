//! Unit tests for `apply_remote_tangent` with the crate's routing.

use super::*;
use crate::client_state::prelude::{RadioSubtype, TangentKind, TangentTransition};
use crate::radio_routing::fixtures::{SW_FREQUENCY, World, at};

fn broadcast(is_pressed: bool) -> TangentBroadcast {
    TangentBroadcast {
        kind: TangentKind::ShortRange,
        is_pressed,
        frequency: SW_FREQUENCY.to_owned(),
        range: 3000,
        range_text: "3000".to_owned(),
        subtype: RadioSubtype::Digital,
        radio_classname: None,
        can_use_sw_radio: true,
        can_use_dd_radio: false,
        nickname: "alpha".to_owned(),
    }
}

#[test]
fn a_press_in_range_is_heard_and_its_release_ends_the_reception() {
    let mut world = World::new();
    world.client("alpha").position = at(1000.0, 0.0, 0.0);
    let now = world.now;
    let pressed = apply_remote_tangent(
        &mut world.session,
        &world.settings,
        &world.antennas,
        &broadcast(true),
        now,
    )
    .unwrap();
    assert_eq!(pressed.transition, TangentTransition::Pressed);
    assert!(pressed.play_start.is_some());
    assert!(
        world
            .session
            .clients
            .own()
            .unwrap()
            .is_receiving_transmission()
    );
    let released = apply_remote_tangent(
        &mut world.session,
        &world.settings,
        &world.antennas,
        &broadcast(false),
        now,
    )
    .unwrap();
    assert!(released.play_end.is_some());
    assert_eq!(released.heard.len(), 1);
    assert!(
        !world
            .session
            .clients
            .own()
            .unwrap()
            .is_receiving_transmission()
    );
}

#[test]
fn a_press_out_of_range_plays_nothing_and_counts_nothing() {
    let mut world = World::new();
    world.client("alpha").position = at(9000.0, 0.0, 0.0);
    let now = world.now;
    let pressed = apply_remote_tangent(
        &mut world.session,
        &world.settings,
        &world.antennas,
        &broadcast(true),
        now,
    )
    .unwrap();
    assert!(pressed.play_start.is_none());
    assert_eq!(pressed.heard, []);
    assert!(!pressed.is_receiving_changed);
}
