//! Unit tests for the killed-player mute matrix.

use super::*;
use crate::client_state::prelude::DataFrame;
use crate::radio_routing::fixtures::{ALPHA, BRAVO, OWN, World};

const GHOST: TeamspeakClientId = TeamspeakClientId(42);
const CHANNEL: [TeamspeakClientId; 4] = [OWN, ALPHA, BRAVO, GHOST];

/// Alpha alive, bravo dead, the ghost unknown.
fn world(is_player_alive: bool) -> World {
    let mut world = World::new();
    world.client("bravo").data_frame = DataFrame::INVALID;
    world.session.game.is_alive = is_player_alive;
    world
}

fn matrix(world: &World, is_serious_mode: bool) -> MuteChanges {
    killed_mute_matrix(&world.context(), is_serious_mode, &CHANNEL)
}

#[test]
fn in_serious_mode_the_living_hear_only_the_living() {
    let changes = matrix(&world(true), true);
    assert_eq!(changes.mute, [BRAVO, GHOST]);
    assert_eq!(changes.unmute, [ALPHA]);
}

#[test]
fn in_serious_mode_the_dead_hear_only_the_dead() {
    let changes = matrix(&world(false), true);
    assert_eq!(changes.mute, [ALPHA]);
    assert_eq!(changes.unmute, [BRAVO, GHOST]);
}

#[test]
fn outside_serious_mode_a_living_player_hears_everyone() {
    let changes = matrix(&world(true), false);
    assert_eq!(changes.mute, []);
    assert_eq!(changes.unmute, [ALPHA, BRAVO, GHOST]);
}

#[test]
fn outside_serious_mode_a_dead_player_hears_everyone() {
    let changes = matrix(&world(false), false);
    assert_eq!(changes.mute, []);
    assert_eq!(changes.unmute, [ALPHA, BRAVO, GHOST]);
}

#[test]
fn disabling_automatic_mute_leaves_mutes_alone_in_serious_mode() {
    let mut world = world(true);
    world.set_setting("disableAutomaticMute", "true");
    assert_eq!(matrix(&world, true), MuteChanges::default());
    world.session.game.is_alive = false;
    assert_eq!(matrix(&world, false).unmute, [ALPHA, BRAVO, GHOST]);
}
