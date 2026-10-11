//! The radio paths of the 1.0 chain: carried short- and long-range radios with towers, diver
//! radios, phones, speaker radios, the intercom with its ducking, and the captured voice.

#![cfg(test)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "signal levels are measured in f64 from i16 samples and handed on as f32 gains"
)]

// `cfg(test)` puts the helpers under the test lint relief of the root `clippy.toml`.
#[cfg(test)]
mod support;

use support::{
    ALPHA, BRAVO, LR_FREQUENCY, SPEAKER_FREQUENCY, SW_FREQUENCY, World, at, channel_rms,
    in_vehicle, is_silent, rms, settled, sine,
};
use tfar_audio::sample_buffer::StereoBuffer;
use tfar_plugin_logic_1_0::client_state::prelude::{
    RadioSubtype, TangentBroadcast, TangentKind, TangentRadio, TeamspeakClientId,
};
use tfar_plugin_logic_1_0::radio_routing::apply_remote_tangent;
use tfar_plugin_logic_1_0::voice_pipeline::VoiceFrameContext;

/// The settled level of `sender` for a sine of `frequency_hz` in a fresh world set up by
/// `setup`, at an amplitude low enough that no stage clips, so levels compare linearly.
fn level(setup: impl Fn(&mut World), sender: TeamspeakClientId, frequency_hz: f64) -> f64 {
    level_at(setup, sender, frequency_hz, 2000.0)
}

/// [`level`] at `amplitude`.
fn level_at(
    setup: impl Fn(&mut World),
    sender: TeamspeakClientId,
    frequency_hz: f64,
    amplitude: f64,
) -> f64 {
    let mut world = World::new();
    setup(&mut world);
    rms(settled(
        &world.play_sine(sender, frequency_hz, amplitude, 16),
        8,
    ))
}

/// `alpha` 100 m north (beyond direct speech) on its short-range radio, as `subtype`.
fn short_range_at_a_hundred_metres(world: &mut World, subtype: RadioSubtype) {
    world.client(ALPHA).position = at(0.0, 100.0, 0.0);
    world.transmit(ALPHA, TangentRadio::ShortRange, SW_FREQUENCY, 1000, subtype);
}

#[test]
fn a_short_range_radio_in_range_is_heard_band_limited() {
    let setup = |world: &mut World| short_range_at_a_hundred_metres(world, RadioSubtype::Digital);
    let pass_band = level(setup, ALPHA, 1500.0);
    let low = level(setup, ALPHA, 200.0);
    let high = level(setup, ALPHA, 7000.0);
    assert!(pass_band > 300.0, "{pass_band}");
    assert!(low < 0.3 * pass_band, "{low} vs {pass_band}");
    assert!(high < 0.3 * pass_band, "{high} vs {pass_band}");
}

#[test]
fn a_short_range_radio_heard_further_away_is_more_distorted() {
    let distortion = |distance: f64| {
        let mut world = World::new();
        world.client(ALPHA).position = at(0.0, distance, 0.0);
        world.transmit(
            ALPHA,
            TangentRadio::ShortRange,
            SW_FREQUENCY,
            1000,
            RadioSubtype::Digital,
        );
        let output = world.play_sine(ALPHA, 1500.0, 8000.0, 16);
        // The ring modulation by the error level moves energy from one frame to the next.
        let levels: Vec<f64> = settled(&output, 8)
            .iter()
            .map(|frame| rms(std::slice::from_ref(frame)))
            .collect();
        let mean = levels.iter().sum::<f64>() / levels.len() as f64;
        levels.iter().map(|level| (level - mean).abs()).sum::<f64>() / mean
    };
    assert!(distortion(900.0) > distortion(100.0));
}

#[test]
fn a_short_range_radio_out_of_range_is_silent() {
    let setup = |world: &mut World| {
        world.client(ALPHA).position = at(0.0, 1500.0, 0.0);
        world.transmit(
            ALPHA,
            TangentRadio::ShortRange,
            SW_FREQUENCY,
            1000,
            RadioSubtype::Digital,
        );
    };
    assert_eq!(level(setup, ALPHA, 1500.0), 0.0);
}

/// `alpha` 5 km east on a long-range radio of 1 km, with the tower or not.
fn long_range_far_away(world: &mut World, with_tower: bool) {
    world.client(ALPHA).position = at(5000.0, 0.0, 0.0);
    world.transmit(
        ALPHA,
        TangentRadio::LongRange,
        LR_FREQUENCY,
        1000,
        RadioSubtype::DigitalLongRange,
    );
    if with_tower {
        // Loss 4800 / 20000 + 200 / 1000 = 0.44.
        world.add_tower(1, at(4800.0, 0.0, 0.0), 20_000.0);
    }
}

#[test]
fn a_long_range_radio_beyond_its_range_is_carried_by_a_tower() {
    let without = level(|world| long_range_far_away(world, false), ALPHA, 1000.0);
    let with = level(|world| long_range_far_away(world, true), ALPHA, 1000.0);
    assert_eq!(without, 0.0);
    assert!(with > 300.0, "{with}");
}

#[test]
fn a_long_range_radio_is_band_limited_to_its_narrow_band() {
    let pass_band = level(|world| long_range_far_away(world, true), ALPHA, 900.0);
    let high = level(|world| long_range_far_away(world, true), ALPHA, 4000.0);
    assert!(high < 0.3 * pass_band, "{high} vs {pass_band}");
}

#[test]
fn an_airborne_radio_is_heard_with_its_own_band() {
    let setup = |world: &mut World| {
        world.client(ALPHA).position = at(0.0, 100.0, 0.0);
        world.transmit(
            ALPHA,
            TangentRadio::LongRange,
            LR_FREQUENCY,
            1000,
            RadioSubtype::Airborne,
        );
    };
    let pass_band = level(setup, ALPHA, 2000.0);
    let low = level(setup, ALPHA, 200.0);
    assert!(pass_band > 300.0, "{pass_band}");
    assert!(low < 0.3 * pass_band, "{low} vs {pass_band}");
}

#[test]
fn a_diver_radio_under_water_is_heard_through_the_underwater_effect() {
    let setup = |world: &mut World| {
        world.client(ALPHA).position = at(0.0, 50.0, -5.0);
        world.transmit(
            ALPHA,
            TangentRadio::ShortRange,
            SW_FREQUENCY,
            1000,
            RadioSubtype::Digital,
        );
        let alpha = world.client(ALPHA);
        alpha.can_use_sw_radio = false;
        alpha.can_use_dd_radio = true;
    };
    let pass_band = level(setup, ALPHA, 1000.0);
    let high = level(setup, ALPHA, 3000.0);
    assert!(pass_band > 300.0, "{pass_band}");
    assert!(high < 0.3 * pass_band, "{high} vs {pass_band}");
}

#[test]
fn an_unknown_subtype_is_silent_on_a_carried_radio() {
    let setup = |world: &mut World| {
        short_range_at_a_hundred_metres(world, RadioSubtype::Unknown("weird".to_owned()));
    };
    assert_eq!(level(setup, ALPHA, 1500.0), 0.0);
}

#[test]
fn a_phone_is_heard_through_its_band_pass() {
    let setup = |world: &mut World| short_range_at_a_hundred_metres(world, RadioSubtype::Phone);
    let pass_band = level(setup, ALPHA, 1850.0);
    let low = level(setup, ALPHA, 150.0);
    assert!(pass_band > 300.0, "{pass_band}");
    assert!(low < 0.1 * pass_band, "{low} vs {pass_band}");
}

#[test]
fn a_lowered_headset_leaves_a_tenth_of_a_carried_radio() {
    let setup = |world: &mut World| short_range_at_a_hundred_metres(world, RadioSubtype::Phone);
    let normal = level(setup, ALPHA, 1850.0);
    let lowered = level(
        |world| {
            setup(world);
            world.set_bool("headsetLowered", true);
        },
        ALPHA,
        1850.0,
    );
    assert!(
        (lowered / normal - 0.1).abs() < 0.01,
        "{}",
        lowered / normal
    );
}

/// `alpha` 200 m north transmitting on the speaker frequency, which only a speaker radio at
/// `speaker` plays.
fn speaker_radio(world: &mut World, speaker: tfar_foundation::geometry::Position, wave_z: f64) {
    world.client(ALPHA).position = at(0.0, 200.0, 0.0);
    world.transmit(
        ALPHA,
        TangentRadio::ShortRange,
        SPEAKER_FREQUENCY,
        1000,
        RadioSubtype::Digital,
    );
    world.add_speaker("speaker_radio", SPEAKER_FREQUENCY, speaker, 7.0, wave_z);
}

#[test]
fn a_speaker_on_the_ground_fades_with_its_distance() {
    let speaker_at = |x: f64| move |world: &mut World| speaker_radio(world, at(x, 0.0, 0.0), 1.0);
    let near = level_at(speaker_at(2.0), ALPHA, 2000.0, 500.0);
    let far = level_at(speaker_at(10.0), ALPHA, 2000.0, 500.0);
    // The speaker's reach is 7 / 10 · 20 = 14 m.
    let expected = tfar_plugin_logic_1_0::voice_pipeline::volume_attenuation(10.0, true, 14.0, 1.0)
        / tfar_plugin_logic_1_0::voice_pipeline::volume_attenuation(2.0, true, 14.0, 1.0);
    assert!(near > 100.0, "{near}");
    assert!(
        (far / near - expected).abs() < 0.1 * expected,
        "{} vs {expected}",
        far / near
    );
}

#[test]
fn a_speaker_to_the_east_is_heard_on_the_right() {
    let mut world = World::new();
    speaker_radio(&mut world, at(5.0, 0.0, 0.0), 1.0);
    let output = world.play_sine(ALPHA, 2000.0, 8000.0, 16);
    let late = settled(&output, 8);
    assert!(channel_rms(late, 1) > 2.0 * channel_rms(late, 0));
}

#[test]
fn a_speaker_under_water_is_low_passed_to_a_murmur() {
    let dry = level(
        |world| speaker_radio(world, at(2.0, 0.0, 0.0), 1.0),
        ALPHA,
        2000.0,
    );
    let wet = level(
        |world| speaker_radio(world, at(2.0, 0.0, 0.0), -2.0),
        ALPHA,
        2000.0,
    );
    assert!(wet < 0.05 * dry, "{wet} vs {dry}");
}

#[test]
fn a_speaker_outside_the_listeners_vehicle_is_muffled_even_when_the_sender_sits_inside() {
    // In the original the sender sharing the listener's vehicle skipped the speaker's muffling.
    let open = level(
        |world| speaker_radio(world, at(3.0, 0.0, 0.0), 1.0),
        ALPHA,
        2000.0,
    );
    let inside = level(
        |world| {
            speaker_radio(world, at(3.0, 0.0, 0.0), 1.0);
            world.client(ALPHA).position = at(0.0, 1.0, 0.0);
            world.client(ALPHA).voice_volume = 0;
            world.client(ALPHA).vehicle = in_vehicle("2:9", 0.5, -1);
            world.session.clients.own_mut().unwrap().vehicle = in_vehicle("2:9", 0.5, -1);
        },
        ALPHA,
        2000.0,
    );
    let expected = tfar_plugin_logic_1_0::voice_pipeline::volume_attenuation(3.0, true, 14.0, 0.5)
        * 0.5_f64.powf(1.2)
        / tfar_plugin_logic_1_0::voice_pipeline::volume_attenuation(3.0, true, 14.0, 1.0);
    // The vehicle low-pass at 2500 Hz takes about 15 % more of the 2 kHz tone.
    assert!(
        inside < 1.2 * expected * open,
        "{inside} vs {open}, expected about {expected}"
    );
    assert!(
        inside > 0.6 * expected * open,
        "{inside} vs {open}, expected about {expected}"
    );
}

/// `alpha` beside the listener in a vehicle on intercom slot 1, holding no tangent, with a
/// silent voice so only the intercom is heard.
fn intercom(world: &mut World) {
    let vehicle = in_vehicle("2:9", 0.3, 1);
    world.client(ALPHA).vehicle = vehicle.clone();
    world.client(ALPHA).voice_volume = 0;
    world.session.clients.own_mut().unwrap().vehicle = vehicle;
}

/// `bravo` presses a long-range tangent the listener hears on their long-range radio.
fn bravo_on_long_range(world: &mut World) {
    world.client(BRAVO).position = at(0.0, 500.0, 0.0);
    let tangent = TangentBroadcast {
        kind: TangentKind::LongRange,
        is_pressed: true,
        frequency: LR_FREQUENCY.to_owned(),
        range: 1000,
        range_text: "1000".to_owned(),
        subtype: RadioSubtype::DigitalLongRange,
        radio_classname: None,
        can_use_sw_radio: false,
        can_use_dd_radio: false,
        nickname: "bravo".to_owned(),
    };
    let effects = apply_remote_tangent(
        &mut world.session,
        &world.settings,
        &world.antennas,
        &tangent,
        world.now,
    )
    .unwrap();
    assert!(effects.is_receiving_changed);
}

#[test]
fn the_intercom_carries_a_sender_in_the_same_vehicle_at_the_intercom_volume() {
    let quiet = level(intercom, ALPHA, 1000.0);
    let loud = level(
        |world| {
            intercom(world);
            world.set_number("intercomVolume", 0.2);
        },
        ALPHA,
        1000.0,
    );
    assert!(quiet > 100.0, "{quiet}");
    assert!((loud / quiet - 2.0).abs() < 0.05, "{}", loud / quiet);
    let off = level(
        |world| {
            intercom(world);
            world.set_bool("intercomEnabled", false);
        },
        ALPHA,
        1000.0,
    );
    assert_eq!(off, 0.0);
}

#[test]
fn the_intercom_is_ducked_while_a_long_range_transmission_is_received() {
    let plain = level(intercom, ALPHA, 1000.0);
    let ducked = level(
        |world| {
            intercom(world);
            bravo_on_long_range(world);
        },
        ALPHA,
        1000.0,
    );
    // intercomDucking 0.2 leaves 0.8.
    assert!((ducked / plain - 0.8).abs() < 0.02, "{}", ducked / plain);
}

#[test]
fn the_captured_voice_plays_on_a_speaker_radio_the_listener_stands_by() {
    let mut world = World::new();
    world.transmit(
        support::OWN,
        TangentRadio::ShortRange,
        SPEAKER_FREQUENCY,
        1000,
        RadioSubtype::Digital,
    );
    world.add_speaker(
        "speaker_radio",
        SPEAKER_FREQUENCY,
        at(3.0, 0.0, 0.0),
        7.0,
        1.0,
    );
    world.ensure(support::OWN);
    let mut heard = Vec::new();
    for index in 0..16 {
        let captured = sine(2000.0, 8000.0, index);
        let context = VoiceFrameContext {
            settings: &world.settings,
            antennas: &world.antennas,
            is_in_game: true,
            is_serious_mode: false,
            sender_metadata: None,
            now: world.now,
        };
        let block = world
            .processor
            .process_own_voice_for_speakers(&mut world.session, &context, &captured)
            .cloned();
        heard.push(block.unwrap());
    }
    assert!(rms(settled(&heard, 8)) > 100.0);
}

#[test]
fn without_speakers_the_captured_voice_is_not_played() {
    let mut world = World::new();
    let captured = sine(2000.0, 8000.0, 0);
    let context = VoiceFrameContext {
        settings: &world.settings,
        antennas: &world.antennas,
        is_in_game: true,
        is_serious_mode: false,
        sender_metadata: None,
        now: world.now,
    };
    let block =
        world
            .processor
            .process_own_voice_for_speakers(&mut world.session, &context, &captured);
    assert!(block.is_none());
}

#[test]
fn the_captured_voice_skips_the_intercom_and_direct_speech() {
    let mut world = World::new();
    intercom(&mut world);
    world.session.clients.own_mut().unwrap().vehicle = in_vehicle("2:9", 0.3, 1);
    world.add_speaker(
        "speaker_radio",
        SPEAKER_FREQUENCY,
        at(3.0, 0.0, 0.0),
        7.0,
        1.0,
    );
    world.ensure(support::OWN);
    let mut heard: Vec<StereoBuffer> = Vec::new();
    for index in 0..8 {
        let captured = sine(1000.0, 8000.0, index);
        let context = VoiceFrameContext {
            settings: &world.settings,
            antennas: &world.antennas,
            is_in_game: true,
            is_serious_mode: false,
            sender_metadata: None,
            now: world.now,
        };
        let block = world
            .processor
            .process_own_voice_for_speakers(&mut world.session, &context, &captured)
            .cloned();
        heard.push(block.unwrap());
    }
    // Not transmitting, the local player is on no speaker; the intercom is skipped.
    assert!(is_silent(&heard));
}
