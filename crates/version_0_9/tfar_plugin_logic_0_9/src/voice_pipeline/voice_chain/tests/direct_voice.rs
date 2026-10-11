//! Scenario tests for the direct voice: the distance gain, the vehicle and can't-speak
//! low-passes and the direction, against `reference/tfar_0_9_12/ts/src/plugin.cpp:2698-2725`.

use crate::voice_pipeline::distance_gain::volume_from_distance;
use crate::voice_pipeline::test_scene::{
    ALPHA, Scene, Signal, at, channel_rms, decibels, rms, vehicle,
};
use crate::voice_pipeline::voice_gains::vehicle_loss_gain;

/// The ILD gain of both ears for a sound straight ahead or straight behind.
const ILD_CENTRE_GAIN: f64 = 0.625;

/// Frames skipped before a level is measured: the filters settle, the HRTF fills its block.
const SETTLE_FRAMES: usize = 4_800;

/// The level of `signal` from `Alpha` after the chain, in a scene `setup` prepares.
fn level_of(signal: Signal, setup: impl FnOnce(&mut Scene)) -> f64 {
    let mut scene = Scene::new();
    setup(&mut scene);
    scene.prepare("Alpha");
    rms(&scene.run(ALPHA, &signal, 40), SETTLE_FRAMES)
}

#[test]
fn a_sender_in_range_is_heard_at_the_distance_gain() {
    let level = level_of(Signal::Tone(1000.0, 8000.0), |_| {});
    let expected =
        8000.0 / 2.0_f64.sqrt() * volume_from_distance(10.0, true, 20, 1.0) * ILD_CENTRE_GAIN;
    assert!(
        (level / expected - 1.0).abs() < 0.01,
        "{level} vs {expected}"
    );
}

#[test]
fn a_sender_beyond_their_voice_volume_is_not_heard_directly() {
    let level = level_of(Signal::Tone(1000.0, 8000.0), |scene| {
        scene.client("Alpha").voice_volume = 9;
    });
    assert_eq!(level, 0.0);
}

#[test]
fn a_vehicle_between_the_two_muffles_the_voice_with_a_low_pass() {
    let in_vehicle = |scene: &mut Scene| {
        scene.client("Alpha").position = at(0.0, 5.0, 0.0);
        scene.client("Me").vehicle = vehicle("car", 0.5);
    };
    let low = level_of(Signal::Tone(300.0, 8000.0), in_vehicle);
    let high = level_of(Signal::Tone(8000.0, 8000.0), in_vehicle);
    let expected_low = 8000.0 / 2.0_f64.sqrt()
        * volume_from_distance(5.0, true, 20, 0.5)
        * vehicle_loss_gain(0.5)
        * ILD_CENTRE_GAIN;
    assert!(
        (low / expected_low - 1.0).abs() < 0.05,
        "{low} vs {expected_low}"
    );
    assert!(
        decibels(high, low) < -20.0,
        "8 kHz is {} dB",
        decibels(high, low)
    );
}

#[test]
fn the_same_vehicle_keeps_the_voice_unmuffled() {
    let unmuffled = level_of(Signal::Tone(8000.0, 8000.0), |scene| {
        scene.client("Me").vehicle = vehicle("car", 0.5);
        scene.client("Alpha").vehicle = vehicle("car", 0.4);
    });
    let expected =
        8000.0 / 2.0_f64.sqrt() * volume_from_distance(10.0, true, 20, 1.0) * ILD_CENTRE_GAIN;
    assert!((unmuffled / expected - 1.0).abs() < 0.01);
}

#[test]
fn a_sender_who_cannot_speak_is_muffled_to_a_hundred_hertz_at_fourteen_times_the_gain() {
    let muffled = |scene: &mut Scene| {
        scene.client("Alpha").position = at(0.0, 3.0, 0.0);
        scene.client("Alpha").can_speak = false;
    };
    let low = level_of(Signal::Tone(30.0, 1000.0), muffled);
    let high = level_of(Signal::Tone(1000.0, 1000.0), muffled);
    let expected_low = 1000.0 / 2.0_f64.sqrt()
        * volume_from_distance(3.0, false, 20, 1.0)
        * 14.0
        * ILD_CENTRE_GAIN;
    assert!(
        (low / expected_low - 1.0).abs() < 0.1,
        "{low} vs {expected_low}"
    );
    assert!(
        decibels(high, low) < -60.0,
        "1 kHz is {} dB",
        decibels(high, low)
    );
}

#[test]
fn a_listener_who_cannot_speak_also_takes_the_muffled_branch() {
    let high = level_of(Signal::Tone(1000.0, 1000.0), |scene| {
        scene.state.can_speak = false;
    });
    let open = level_of(Signal::Tone(1000.0, 1000.0), |_| {});
    assert!(decibels(high, open) < -60.0);
}

#[test]
fn a_sender_to_the_right_is_louder_in_the_right_ear_through_the_hrtf() {
    let mut scene = Scene::with_hrtf();
    scene.client("Alpha").position = at(10.0, 0.0, 0.0);
    scene.prepare("Alpha");
    let output = scene.run(ALPHA, &Signal::Noise(8000.0), 60);
    let (left, right) = channel_rms(&output, SETTLE_FRAMES);
    assert!(
        decibels(right, left) > 6.0,
        "right over left {} dB",
        decibels(right, left)
    );
}

#[test]
fn without_an_hrtf_source_the_ild_places_the_sender() {
    let mut scene = Scene::new();
    scene.client("Alpha").position = at(10.0, 0.0, 0.0);
    scene.prepare("Alpha");
    let (left, right) = channel_rms(&scene.run(ALPHA, &Signal::Tone(1000.0, 8000.0), 20), 960);
    assert!(
        (right / left - 4.0).abs() < 0.05,
        "right over left {}",
        right / left
    );
}

#[test]
fn the_listeners_view_angle_turns_the_direction() {
    let mut scene = Scene::new();
    scene.client("Me").view_angle = crate::client_state::prelude::Angle::from_degrees(90.0);
    scene.prepare("Alpha");
    let (left, right) = channel_rms(&scene.run(ALPHA, &Signal::Tone(1000.0, 8000.0), 20), 960);
    assert!(
        (left / right - 4.0).abs() < 0.05,
        "left over right {}",
        left / right
    );
}

#[test]
fn the_makeup_gain_keeps_speech_straight_ahead_at_its_input_level() {
    let mut scene = Scene::with_hrtf();
    scene.client("Alpha").position = at(0.0, 1.0, 0.0);
    scene.prepare("Alpha");
    let signal = Signal::Noise(6000.0);
    let output = scene.run(ALPHA, &signal, 100);
    let input_level = 6000.0 / 3.0_f64.sqrt();
    let level_db = decibels(rms(&output, SETTLE_FRAMES), input_level);
    assert!(level_db.abs() < 2.5, "straight ahead at {level_db} dB");
}
