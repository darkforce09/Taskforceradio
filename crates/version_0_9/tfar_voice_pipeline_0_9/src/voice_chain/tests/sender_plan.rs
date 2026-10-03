//! Unit tests for `plan_sender`: the numbers the stereo function computes per frame, against
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:2687-2797`.

use tfar_client_state_0_9::prelude::{RadioSubtype, TangentKind};

use super::*;
use crate::test_scene::{FREQUENCY, Scene, at};

/// The plan of `Alpha`'s frame for `Me`, with its receptions.
fn plan(scene: &Scene) -> (SenderPlan, Vec<PlannedRoute>) {
    let server = &scene.state;
    let mut routes = Vec::new();
    let plan = plan_sender(
        server,
        "Alpha",
        server.client("Alpha").unwrap(),
        server.client("Me").unwrap(),
        &mut Vec::new(),
        &mut routes,
    );
    (plan, routes)
}

#[test]
fn the_listener_hears_no_direct_voice_of_their_own() {
    let scene = Scene::new();
    let server = &scene.state;
    let me = server.client("Me").unwrap();
    let plan = plan_sender(server, "Me", me, me, &mut Vec::new(), &mut Vec::new());
    assert_eq!(plan.direct_voice, None);
}

#[test]
fn the_short_range_error_is_the_effective_distance_over_the_range() {
    let mut scene = Scene::new();
    scene.client("Alpha").position = at(0.0, 100.0, 0.0);
    scene.client("Alpha").terrain_interception = 2;
    scene.transmit(
        "Alpha",
        TangentKind::ShortRange,
        RadioSubtype::Digital,
        1000,
    );
    let (plan, routes) = plan(&scene);
    assert_eq!(plan.effect, RadioEffectKind::Personal);
    let [route] = routes.as_slice() else {
        panic!("one reception expected, got {routes:?}");
    };
    let effective = 100.0 + 14.0 + 14.0 * 100.0 / 2000.0;
    assert!((route.effect_error - effective / 1000.0).abs() < 1e-12);
    assert!((route.effect_gain - 0.4096 * 0.35).abs() < 1e-12);
    assert_eq!(route.loudspeaker, None);
}

#[test]
fn the_underwater_error_is_the_raw_distance_over_the_diver_reach_at_the_diver_volume() {
    let mut scene = Scene::new();
    scene.client("Alpha").position = at(0.0, 30.0, 0.0);
    scene.client("Alpha").terrain_interception = 5;
    scene.transmit(
        "Alpha",
        TangentKind::DiverDigital,
        RadioSubtype::DiverDigital,
        1000,
    );
    scene.state.dd_frequency = FREQUENCY.to_owned();
    scene.state.dd_volume_level = 4;
    scene.state.waves_level = 0.5;
    let (plan, routes) = plan(&scene);
    assert_eq!(plan.effect, RadioEffectKind::UnderWater);
    let [route] = routes.as_slice() else {
        panic!("one reception expected, got {routes:?}");
    };
    assert!((route.effect_error - 30.0 / 185.0).abs() < 1e-12);
    assert!((route.effect_gain - 0.5_f64.powi(4) * 0.6).abs() < 1e-12);
}

#[test]
fn a_phone_plays_at_ten_times_the_volume_and_an_unknown_subtype_at_none() {
    let mut scene = Scene::new();
    scene.transmit("Alpha", TangentKind::ShortRange, RadioSubtype::Phone, 1000);
    let (_, routes) = plan(&scene);
    assert!((routes[0].effect_gain - 4.096).abs() < 1e-12);
    scene.client("Alpha").subtype = RadioSubtype::Unknown(String::new());
    let (plan, routes) = plan(&scene);
    assert_eq!(plan.effect, RadioEffectKind::Silent);
    assert_eq!(routes[0].effect_gain, 0.0);
}

#[test]
fn a_loudspeaker_reach_is_truncated_to_whole_metres() {
    let mut scene = Scene::new();
    scene.state.sw_frequencies.clear();
    scene.state.speaker_distance = 25.0;
    scene.transmit(
        "Alpha",
        TangentKind::ShortRange,
        RadioSubtype::Digital,
        1000,
    );
    scene.add_loudspeaker(at(0.0, 5.0, 0.0), 3, 1.0);
    let (_, routes) = plan(&scene);
    let loudspeaker = routes[0].loudspeaker.unwrap();
    let expected = volume_from_distance(5.0, true, 7, 1.0);
    assert_eq!(loudspeaker.level, DistanceLevel::Plain { gain: expected });
    assert!(!loudspeaker.is_under_water);
}

#[test]
fn a_loudspeaker_carries_five_metres_when_the_sender_cannot_speak() {
    let mut scene = Scene::new();
    scene.state.sw_frequencies.clear();
    scene.client("Alpha").can_speak = false;
    scene.transmit(
        "Alpha",
        TangentKind::ShortRange,
        RadioSubtype::Digital,
        1000,
    );
    scene.add_loudspeaker(at(0.0, 6.0, 0.0), 10, -2.0);
    let (_, routes) = plan(&scene);
    let loudspeaker = routes[0].loudspeaker.unwrap();
    assert_eq!(loudspeaker.level, DistanceLevel::Plain { gain: 0.0 });
    assert!(loudspeaker.is_under_water);
}

#[test]
fn a_muffled_direct_voice_carries_the_vehicle_loss() {
    let mut scene = Scene::new();
    scene.client("Alpha").position = at(0.0, 5.0, 0.0);
    scene.client("Alpha").vehicle = crate::test_scene::vehicle("tank", 0.7);
    let (plan, _) = plan(&scene);
    let level = plan.direct_voice.unwrap().level;
    let expected = volume_from_distance(5.0, true, 20, 1.0 - 0.7) * vehicle_loss_gain(0.7);
    assert_eq!(
        level,
        DirectVoiceLevel::Heard(DistanceLevel::VehicleMuffled { gain: expected })
    );
}

#[test]
fn a_direct_voice_that_cannot_be_heard_takes_the_cant_speak_level() {
    let mut scene = Scene::new();
    scene.client("Alpha").position = at(0.0, 2.0, 0.0);
    scene.client("Alpha").vehicle = crate::test_scene::vehicle("tank", 0.7);
    scene.client("Alpha").can_speak = false;
    let (plan, _) = plan(&scene);
    let expected = volume_from_distance(2.0, false, 20, 1.0) * CANT_SPEAK_GAIN;
    assert_eq!(
        plan.direct_voice.unwrap().level,
        DirectVoiceLevel::CantSpeak { gain: expected }
    );
}
