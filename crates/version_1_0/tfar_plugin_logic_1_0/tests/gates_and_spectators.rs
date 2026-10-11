//! The gates in front of the 1.0 voice chain: TFAR state, serious mode and the spectator rules.

#![cfg(test)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "signal levels are measured in f64 from i16 samples and handed on as f32 gains"
)]

// `cfg(test)` puts the helpers under the test lint relief of the root `clippy.toml`.
#[cfg(test)]
mod support;

use std::time::Duration;

use support::{ALPHA, OWN, SW_FREQUENCY, World, at, is_silent, rms, sine};
use tfar_plugin_logic_1_0::client_state::prelude::{
    PLUGIN_ENABLED_CACHE, RadioSubtype, TangentRadio, TeamspeakClientId,
};
use tfar_plugin_logic_1_0::voice_pipeline::{VoiceFrameContext, VoiceOutcome};

#[test]
fn a_sender_without_tfar_passes_through_untouched() {
    let mut world = World::new();
    let stranger = TeamspeakClientId(9);
    world.session.clients.client_joined(stranger, "stranger");
    let input = sine(1000.0, 8000.0, 0);
    let (outcome, output) = world.process(stranger, &input);
    assert_eq!(outcome, VoiceOutcome::PassThrough);
    assert_eq!(output, input);
}

#[test]
fn a_sender_whose_metadata_says_not_playing_passes_through() {
    let mut world = World::new();
    let stranger = TeamspeakClientId(9);
    world.session.clients.client_joined(stranger, "stranger");
    let input = sine(1000.0, 8000.0, 0);
    let mut samples = input.clone();
    let context = VoiceFrameContext {
        settings: &world.settings,
        antennas: &world.antennas,
        is_in_game: true,
        is_serious_mode: false,
        sender_metadata: Some("Connected to Game: [B]No[/B]"),
        now: world.now,
    };
    let outcome =
        world
            .processor
            .process_sender(&mut world.session, &context, stranger, &mut samples);
    assert_eq!(outcome, VoiceOutcome::PassThrough);
    assert_eq!(samples, input);
}

#[test]
fn an_unknown_sender_passes_through() {
    let mut world = World::new();
    let input = sine(1000.0, 8000.0, 0);
    let (outcome, output) = world.process(TeamspeakClientId(42), &input);
    assert_eq!(outcome, VoiceOutcome::PassThrough);
    assert_eq!(output, input);
}

#[test]
fn outside_a_game_every_frame_passes_through() {
    let mut world = World::new();
    let input = sine(1000.0, 8000.0, 0);
    let mut samples = input.clone();
    let context = VoiceFrameContext {
        settings: &world.settings,
        antennas: &world.antennas,
        is_in_game: false,
        is_serious_mode: false,
        sender_metadata: None,
        now: world.now,
    };
    let outcome = world
        .processor
        .process_sender(&mut world.session, &context, ALPHA, &mut samples);
    assert_eq!(outcome, VoiceOutcome::PassThrough);
    assert_eq!(samples, input);
}

#[test]
fn the_local_players_own_voice_without_tfar_is_silence() {
    let mut world = World::new();
    // Past the plugin-enabled cache, with no metadata to read, the local player has no TFAR.
    world.now += PLUGIN_ENABLED_CACHE + Duration::from_secs(1);
    let (outcome, output) = world.process(OWN, &sine(1000.0, 8000.0, 0));
    assert_eq!(outcome, VoiceOutcome::Silenced);
    assert!(is_silent(&[output]));
}

/// The local player spectates and the sender, an enemy, is kept from spectators.
fn spectating_listener_with_an_enemy_sender(world: &mut World) {
    world.session.clients.own_mut().unwrap().is_spectating = true;
    world.client(ALPHA).is_enemy_to_player = true;
    world.client(ALPHA).position = at(0.0, 5.0, 0.0);
    world.set_bool("spectatorNotHearEnemies", true);
}

#[test]
fn a_dead_listener_in_serious_mode_is_silenced_for_a_sender_kept_from_spectators() {
    let mut world = World::new();
    spectating_listener_with_an_enemy_sender(&mut world);
    world.session.game.is_alive = false;
    world.is_serious_mode = true;
    let (outcome, output) = world.process(ALPHA, &sine(1000.0, 8000.0, 0));
    assert_eq!(outcome, VoiceOutcome::Silenced);
    assert!(is_silent(&[output]));
}

#[test]
fn outside_serious_mode_a_sender_kept_from_spectators_is_processed_to_silence() {
    let mut world = World::new();
    spectating_listener_with_an_enemy_sender(&mut world);
    world.session.game.is_alive = false;
    let output = world.play_sine(ALPHA, 1000.0, 8000.0, 3);
    assert!(is_silent(&output));
}

#[test]
fn mute_spectators_silences_one_spectator_for_another() {
    let mut world = World::new();
    world.session.clients.own_mut().unwrap().is_spectating = true;
    world.client(ALPHA).is_spectating = true;
    world.set_bool("muteSpectators", true);
    let (outcome, output) = world.process(ALPHA, &sine(1000.0, 8000.0, 0));
    assert_eq!(outcome, VoiceOutcome::Silenced);
    assert!(is_silent(&[output]));
}

#[test]
fn two_spectators_hear_each_other_plainly() {
    let mut world = World::new();
    world.session.clients.own_mut().unwrap().is_spectating = true;
    world.client(ALPHA).is_spectating = true;
    world.client(ALPHA).position = at(0.0, 500.0, 0.0);
    let input = sine(1000.0, 8000.0, 0);
    world.ensure(ALPHA);
    let (outcome, output) = world.process(ALPHA, &input);
    assert_eq!(outcome, VoiceOutcome::Processed);
    assert_eq!(output, input);
}

#[test]
fn a_spectating_listener_does_not_hear_friendlies_when_the_setting_says_so() {
    let mut world = World::new();
    world.session.clients.own_mut().unwrap().is_spectating = true;
    world.client(ALPHA).position = at(0.0, 5.0, 0.0);
    world.set_bool("spectatorCanHearFriendlies", false);
    let output = world.play_sine(ALPHA, 1000.0, 8000.0, 3);
    assert!(is_silent(&output));
}

#[test]
fn a_spectating_listener_hears_friendlies_by_default() {
    let mut world = World::new();
    world.session.clients.own_mut().unwrap().is_spectating = true;
    world.client(ALPHA).position = at(0.0, 5.0, 0.0);
    let output = world.play_sine(ALPHA, 1000.0, 8000.0, 3);
    assert!(rms(&output) > 100.0);
}

#[test]
fn a_spectating_sender_is_heard_neither_directly_nor_over_radio() {
    let mut world = World::new();
    world.client(ALPHA).is_spectating = true;
    world.client(ALPHA).position = at(0.0, 5.0, 0.0);
    world.transmit(
        ALPHA,
        TangentRadio::ShortRange,
        SW_FREQUENCY,
        1000,
        RadioSubtype::Digital,
    );
    let output = world.play_sine(ALPHA, 1000.0, 8000.0, 8);
    assert!(is_silent(&output));
}

#[test]
fn the_voice_volume_multiplier_and_the_global_volume_both_scale_the_voice() {
    let mut reference = World::new();
    reference.client(ALPHA).position = at(0.0, 5.0, 0.0);
    let reference_rms = rms(&reference.play_sine(ALPHA, 1000.0, 8000.0, 3));
    let mut scaled = World::new();
    scaled.client(ALPHA).position = at(0.0, 5.0, 0.0);
    scaled.client(ALPHA).voice_volume_multiplier = 0.5;
    scaled.session.game.global_volume = 0.5;
    let scaled_rms = rms(&scaled.play_sine(ALPHA, 1000.0, 8000.0, 3));
    let ratio = scaled_rms / reference_rms;
    assert!((ratio - 0.25).abs() < 0.01, "{ratio}");
}
