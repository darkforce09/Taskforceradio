//! Unit tests for `mute_decision`.

use super::*;
use crate::fixtures::{SW_FREQUENCY, World, at};
use tfar_client_state_1_0::prelude::{DataFrame, TangentRadio};

const MUTE: Option<MuteDecision> = Some(MuteDecision {
    mute: true,
    reset_voices: true,
});
const UNMUTE: Option<MuteDecision> = Some(MuteDecision {
    mute: false,
    reset_voices: false,
});

fn decide(world: &World, nickname: &str, is_serious_mode: bool) -> Option<MuteDecision> {
    let target = world.session.clients.get(nickname);
    mute_decision(&world.context(), target, is_serious_mode, None)
}

fn far_away_world() -> World {
    let mut world = World::new();
    let alpha = world.client("alpha");
    alpha.voice_volume = 20;
    alpha.position = at(100.0, 0.0, 0.0);
    world
}

#[test]
fn outside_serious_mode_every_other_client_is_unmuted() {
    let world = far_away_world();
    assert_eq!(decide(&world, "alpha", false), UNMUTE);
    assert_eq!(decide(&world, "me", false), None);
}

#[test]
fn disabling_automatic_mute_unmutes() {
    let mut world = far_away_world();
    world.set_setting("disableAutomaticMute", "true");
    assert_eq!(decide(&world, "alpha", true), UNMUTE);
}

#[test]
fn the_local_player_is_never_muted() {
    let world = far_away_world();
    assert_eq!(decide(&world, "me", true), None);
}

#[test]
fn a_client_within_voice_range_and_margin_is_heard() {
    let mut world = far_away_world();
    world.client("alpha").position = at(35.0, 0.0, 0.0);
    assert_eq!(decide(&world, "alpha", true), UNMUTE);
}

#[test]
fn a_client_beyond_voice_range_and_not_on_radio_is_muted() {
    let world = far_away_world();
    assert_eq!(decide(&world, "alpha", true), MUTE);
}

#[test]
fn a_client_beyond_voice_range_heard_on_radio_stays_unmuted() {
    let mut world = far_away_world();
    world.transmit(
        "alpha",
        TangentRadio::ShortRange,
        SW_FREQUENCY,
        3000,
        at(100.0, 0.0, 0.0),
    );
    assert_eq!(decide(&world, "alpha", true), UNMUTE);
    let target = world.session.clients.get("alpha");
    assert_eq!(
        mute_decision(&world.context(), target, true, Some(false)),
        MUTE
    );
}

#[test]
fn a_dead_client_is_muted_for_a_living_player() {
    let mut world = World::new();
    world.client("alpha").data_frame = DataFrame::INVALID;
    assert_eq!(decide(&world, "alpha", true), MUTE);
}

#[test]
fn a_spectating_player_judges_dead_clients_by_distance() {
    let mut world = World::new();
    world.client("alpha").data_frame = DataFrame::INVALID;
    world.session.clients.own_mut().unwrap().is_spectating = true;
    assert_eq!(decide(&world, "alpha", true), UNMUTE);
}

#[test]
fn an_unknown_client_is_muted_without_a_voice_reset() {
    let world = World::new();
    assert_eq!(
        mute_decision(&world.context(), None, true, None),
        Some(MuteDecision {
            mute: true,
            reset_voices: false
        })
    );
}
