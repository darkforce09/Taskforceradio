//! Scenario tests for the radio receptions: each effect by subtype, the ear, and the
//! loudspeaker stages, against `reference/tfar_0_9_12/ts/src/plugin.cpp:2728-2794`.

use tfar_client_state_0_9::FrequencySettings;
use tfar_client_state_0_9::prelude::{RadioSubtype, StereoMode, TangentKind};

use crate::test_scene::{ALPHA, FREQUENCY, Scene, Signal, at, channel_rms, decibels, rms, vehicle};

/// Frames skipped before a level is measured: the delay line, the filters and the HRTF settle.
const SETTLE_FRAMES: usize = 4_800;

/// A scene where `Alpha` stands 100 m away, beyond their voice, and transmits on [`FREQUENCY`]
/// with a radio of `subtype` over `kind` and 1000 m range; `setup` changes the rest.
fn radio_scene(kind: TangentKind, subtype: RadioSubtype, setup: impl FnOnce(&mut Scene)) -> Scene {
    let mut scene = Scene::new();
    scene.client("Alpha").position = at(0.0, 100.0, 0.0);
    scene.transmit("Alpha", kind, subtype, 1000);
    setup(&mut scene);
    scene.prepare("Alpha");
    scene
}

/// The level of a `hz` tone sent over a fresh [`radio_scene`].
fn radio_level(
    hz: f64,
    kind: TangentKind,
    subtype: RadioSubtype,
    setup: impl FnOnce(&mut Scene),
) -> f64 {
    let mut scene = radio_scene(kind, subtype, setup);
    rms(
        &scene.run(ALPHA, &Signal::Tone(hz, 8000.0), 40),
        SETTLE_FRAMES,
    )
}

/// A short-range `digital` transmission's level at `hz`.
fn short_range_level(hz: f64) -> f64 {
    radio_level(hz, TangentKind::ShortRange, RadioSubtype::Digital, |_| {})
}

#[test]
fn a_short_range_radio_in_range_is_heard_band_limited_by_the_personal_effect() {
    let voice_band = short_range_level(1500.0);
    let bass = short_range_level(150.0);
    let treble = short_range_level(9000.0);
    assert!(voice_band > 300.0, "voice band at {voice_band}");
    assert!(
        decibels(bass, voice_band) < -20.0,
        "150 Hz at {} dB",
        decibels(bass, voice_band)
    );
    assert!(
        decibels(treble, voice_band) < -20.0,
        "9 kHz at {} dB",
        decibels(treble, voice_band)
    );
}

#[test]
fn a_short_range_radio_out_of_range_is_silent() {
    let level = radio_level(
        1500.0,
        TangentKind::ShortRange,
        RadioSubtype::Digital,
        |scene| {
            scene.client("Alpha").range = 50;
        },
    );
    assert_eq!(level, 0.0);
}

#[test]
fn a_long_range_radio_on_the_listeners_long_range_set_has_the_narrow_long_range_band() {
    let on_long_range = |scene: &mut Scene| {
        scene.client("Alpha").position = at(0.0, 8000.0, 0.0);
        scene.client("Alpha").range = 20_000;
        scene.state.sw_frequencies.clear();
        scene.state.lr_frequencies.insert(
            FREQUENCY.to_owned(),
            FrequencySettings {
                volume: 7,
                stereo_mode: StereoMode::Both,
            },
        );
    };
    let lr = TangentKind::LongRange;
    let voice_band = radio_level(800.0, lr, RadioSubtype::DigitalLongRange, on_long_range);
    let treble = radio_level(5000.0, lr, RadioSubtype::DigitalLongRange, on_long_range);
    assert!(voice_band > 100.0, "voice band at {voice_band}");
    assert!(
        decibels(treble, voice_band) < -15.0,
        "5 kHz at {} dB",
        decibels(treble, voice_band)
    );
}

#[test]
fn a_diver_radio_under_water_has_the_underwater_band_pass() {
    let diving = |scene: &mut Scene| {
        scene.client("Alpha").position = at(0.0, 30.0, 0.0);
        scene.state.dd_frequency = FREQUENCY.to_owned();
        scene.state.dd_volume_level = 7;
    };
    let dd = TangentKind::DiverDigital;
    let centre = radio_level(1000.0, dd, RadioSubtype::DiverDigital, diving);
    let outside = radio_level(3000.0, dd, RadioSubtype::DiverDigital, diving);
    assert!(centre > 300.0, "1 kHz at {centre}");
    assert!(
        decibels(outside, centre) < -20.0,
        "3 kHz at {} dB",
        decibels(outside, centre)
    );
}

#[test]
fn a_phone_has_the_earpiece_band_pass_at_ten_times_the_volume() {
    let sr = TangentKind::ShortRange;
    // Below the compressor threshold (65 dB, about 1778 on the sample scale), so the level is
    // the band-pass and the gain alone.
    let quiet_level = |hz: f64| {
        let mut scene = radio_scene(sr, RadioSubtype::Phone, |_| {});
        rms(
            &scene.run(ALPHA, &Signal::Tone(hz, 1500.0), 40),
            SETTLE_FRAMES,
        )
    };
    let centre = quiet_level(1850.0);
    let bass = quiet_level(100.0);
    let expected = 1500.0 / 2.0_f64.sqrt() * 0.4096 * 10.0;
    assert!(
        (centre / expected - 1.0).abs() < 0.15,
        "1850 Hz at {centre} for {expected}"
    );
    assert!(
        decibels(bass, centre) < -20.0,
        "100 Hz at {} dB",
        decibels(bass, centre)
    );
}

#[test]
fn an_unknown_subtype_is_silent() {
    let level = radio_level(
        1500.0,
        TangentKind::ShortRange,
        RadioSubtype::Unknown("tf_toaster".to_owned()),
        |_| {},
    );
    assert_eq!(level, 0.0);
}

#[test]
fn a_radio_on_the_left_ear_leaves_the_right_ear_silent() {
    let mut scene = radio_scene(TangentKind::ShortRange, RadioSubtype::Digital, |scene| {
        scene.state.sw_frequencies.insert(
            FREQUENCY.to_owned(),
            FrequencySettings {
                volume: 7,
                stereo_mode: StereoMode::Left,
            },
        );
    });
    let (left, right) = channel_rms(&scene.run(ALPHA, &Signal::Tone(1500.0, 8000.0), 20), 960);
    assert!(left > 300.0);
    assert_eq!(right, 0.0);
}

/// A scene where the listener has no radio of their own and hears `Alpha` only on `Bravo`'s
/// loudspeaker at `position`, volume 10 (20 m reach), wave height `wave_z`.
fn loudspeaker_scene(position: (f64, f64, f64), wave_z: f64) -> Scene {
    radio_scene(TangentKind::ShortRange, RadioSubtype::Digital, |scene| {
        scene.state.sw_frequencies.clear();
        scene.add_loudspeaker(at(position.0, position.1, position.2), 10, wave_z);
    })
}

#[test]
fn a_loudspeaker_on_the_ground_is_heard_within_its_reach_and_not_beyond() {
    let mut near = loudspeaker_scene((0.0, 5.0, 0.0), 1.0);
    let near_level = rms(
        &near.run(ALPHA, &Signal::Tone(2000.0, 8000.0), 40),
        SETTLE_FRAMES,
    );
    let mut far = loudspeaker_scene((0.0, 25.0, 0.0), 1.0);
    let far_level = rms(
        &far.run(ALPHA, &Signal::Tone(2000.0, 8000.0), 40),
        SETTLE_FRAMES,
    );
    assert!(near_level > 300.0, "5 m at {near_level}");
    assert_eq!(far_level, 0.0);
}

#[test]
fn a_loudspeaker_to_the_right_is_placed_to_the_right() {
    let mut scene = loudspeaker_scene((5.0, 0.0, 0.0), 1.0);
    let (left, right) = channel_rms(&scene.run(ALPHA, &Signal::Tone(2000.0, 8000.0), 40), 960);
    assert!(
        (right / left - 4.0).abs() < 0.1,
        "right over left {}",
        right / left
    );
}

#[test]
fn a_loudspeaker_to_the_right_is_louder_in_the_right_ear_through_the_hrtf() {
    let mut scene = Scene::with_hrtf();
    scene.client("Alpha").position = at(0.0, 100.0, 0.0);
    scene.transmit(
        "Alpha",
        TangentKind::ShortRange,
        RadioSubtype::Digital,
        1000,
    );
    scene.state.sw_frequencies.clear();
    scene.add_loudspeaker(at(5.0, 0.0, 0.0), 10, 1.0);
    scene.prepare("Alpha");
    let output = scene.run(ALPHA, &Signal::Noise(8000.0), 60);
    let (left, right) = channel_rms(&output, SETTLE_FRAMES);
    assert!(
        decibels(right, left) > 6.0,
        "right over left {} dB",
        decibels(right, left)
    );
}

/// The level of `Alpha`'s noise on `Bravo`'s loudspeaker straight ahead at 5 m, placed by the
/// HRTF when `with_hrtf`, else by the simple ILD.
fn loudspeaker_ahead_level(with_hrtf: bool) -> f64 {
    let mut scene = if with_hrtf {
        Scene::with_hrtf()
    } else {
        Scene::new()
    };
    scene.client("Alpha").position = at(0.0, 100.0, 0.0);
    scene.transmit(
        "Alpha",
        TangentKind::ShortRange,
        RadioSubtype::Digital,
        1000,
    );
    scene.state.sw_frequencies.clear();
    scene.add_loudspeaker(at(0.0, 5.0, 0.0), 10, 1.0);
    scene.prepare("Alpha");
    rms(
        &scene.run(ALPHA, &Signal::Noise(8000.0), 100),
        SETTLE_FRAMES,
    )
}

#[test]
fn a_loudspeaker_through_the_hrtf_gets_no_makeup_gain() {
    // The ILD puts a sound straight ahead at 0.625 (-4.1 dB) in both ears, about the level the
    // diffuse-field sphere gives it; with the +4 dB speech makeup the HRTF would be 4 dB louder.
    let difference = decibels(
        loudspeaker_ahead_level(true),
        loudspeaker_ahead_level(false),
    );
    assert!(
        difference.abs() < 2.0,
        "the HRTF loudspeaker is {difference} dB from the ILD one"
    );
}

#[test]
fn a_loudspeaker_under_water_is_muffled_to_a_hundred_hertz() {
    let mut dry = loudspeaker_scene((0.0, 5.0, 0.0), 1.0);
    let dry_level = rms(
        &dry.run(ALPHA, &Signal::Tone(2000.0, 8000.0), 40),
        SETTLE_FRAMES,
    );
    let mut wet = loudspeaker_scene((0.0, 5.0, 0.0), -2.0);
    let wet_level = rms(
        &wet.run(ALPHA, &Signal::Tone(2000.0, 8000.0), 40),
        SETTLE_FRAMES,
    );
    assert!(
        decibels(wet_level, dry_level) < -40.0,
        "{} dB",
        decibels(wet_level, dry_level)
    );
}

#[test]
fn a_loudspeaker_in_another_vehicle_is_muffled_and_quieter() {
    let mut open = loudspeaker_scene((0.0, 5.0, 0.0), 1.0);
    let open_level = rms(
        &open.run(ALPHA, &Signal::Tone(2000.0, 8000.0), 40),
        SETTLE_FRAMES,
    );
    let mut muffled = radio_scene(TangentKind::ShortRange, RadioSubtype::Digital, |scene| {
        scene.state.sw_frequencies.clear();
        scene.add_loudspeaker(at(0.0, 5.0, 0.0), 10, 1.0);
        scene.state.speakers.get_mut(FREQUENCY).unwrap()[0].vehicle = vehicle("truck", 0.5);
    });
    let muffled_level = rms(
        &muffled.run(ALPHA, &Signal::Tone(2000.0, 8000.0), 40),
        SETTLE_FRAMES,
    );
    let ratio = muffled_level / open_level;
    assert!((0.1..0.4).contains(&ratio), "muffled over open {ratio}");
}

#[test]
fn the_direct_voice_and_the_radio_mix_together() {
    let mut scene = Scene::new();
    scene.transmit(
        "Alpha",
        TangentKind::ShortRange,
        RadioSubtype::Digital,
        1000,
    );
    scene.prepare("Alpha");
    let noise = Signal::Noise(8000.0);
    let both = rms(&scene.run(ALPHA, &noise, 40), SETTLE_FRAMES);
    let mut direct_only = Scene::new();
    direct_only.prepare("Alpha");
    let direct = rms(&direct_only.run(ALPHA, &noise, 40), SETTLE_FRAMES);
    let mut radio_only = Scene::new();
    radio_only.client("Alpha").voice_volume = 5;
    radio_only.transmit(
        "Alpha",
        TangentKind::ShortRange,
        RadioSubtype::Digital,
        1000,
    );
    radio_only.prepare("Alpha");
    let radio = rms(&radio_only.run(ALPHA, &noise, 40), SETTLE_FRAMES);
    assert!(
        both > direct.max(radio),
        "{both} against {direct} and {radio}"
    );
}
