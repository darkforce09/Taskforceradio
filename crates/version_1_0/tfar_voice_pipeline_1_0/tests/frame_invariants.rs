//! What holds for every frame of the 1.0 chain: each compressor advances once, the scratch never
//! grows, and the DSP objects are created once, off the audio path.

#![cfg(test)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "signal levels are measured in f64 from i16 samples and handed on as f32 gains"
)]

// `cfg(test)` puts the helpers under the test lint relief of the root `clippy.toml`.
#[cfg(test)]
mod support;

use std::sync::Arc;

use support::{ALPHA, BRAVO, FRAMES, SPEAKER_FREQUENCY, SW_FREQUENCY, World, at, facing, sine};
use tfar_client_state_1_0::prelude::{LOCAL_RADIO_ID, RadioSubtype, TangentRadio};
use tfar_dynamics::{Compressor, CompressorSettings};
use tfar_sample_buffer::{StereoBuffer, apply_stereo_gain};
use tfar_spatial_audio::{DistanceCurve, ILD_COEFFICIENT_1_0, stereo_pan_gains};
use tfar_voice_pipeline_1_0::{ensure_every_sender_dsp, volume_attenuation};

/// `alpha` 3 m north, heard directly and on the listener's short-range radio.
fn direct_and_radio(world: &mut World) {
    world.client(ALPHA).position = at(0.0, 3.0, 0.0);
    world.transmit(
        ALPHA,
        TangentRadio::ShortRange,
        SW_FREQUENCY,
        1000,
        RadioSubtype::Digital,
    );
}

/// The direct-speech signal of `alpha` 3 m north before its compressor, without HRTF: the input
/// at the voice curve level through the simple ILD.
fn direct_speech_before_compression(input: &StereoBuffer) -> StereoBuffer {
    let curve =
        DistanceCurve::tfar_voice(20.0, |metres| volume_attenuation(metres, true, 20.0, 1.0))
            .unwrap();
    let level = curve.gain_at(3.0, 20.0) as f32;
    let (left, right) = stereo_pan_gains(
        at(0.0, 0.0, 0.0).direction_to(at(0.0, 3.0, 0.0)),
        facing(0.0, 1.0, 0.0).view_angle(),
        ILD_COEFFICIENT_1_0,
    );
    let mut direct = input.clone();
    apply_stereo_gain(&mut direct, left * level, right * level);
    direct
}

#[test]
fn direct_speech_and_radio_each_advance_their_own_compressor_once_per_frame() {
    let mut world = World::new();
    direct_and_radio(&mut world);
    world.ensure(ALPHA);
    let mut direct_reference = Compressor::new(CompressorSettings::VERSION_1_0).unwrap();
    let mut radio_reference = Compressor::new(CompressorSettings::VERSION_1_0).unwrap();
    let mut shared_instance = Compressor::new(CompressorSettings::VERSION_1_0).unwrap();
    for index in 0..3 {
        let input = sine(500.0, 30_000.0, index);
        world.process(ALPHA, &input);
        let mut direct = direct_speech_before_compression(&input);
        let mut radio = input.clone();
        direct_reference.process_stereo_i16(&mut direct.clone());
        radio_reference.process_stereo_i16(&mut radio.clone());
        shared_instance.process_stereo_i16(&mut direct);
        shared_instance.process_stereo_i16(&mut radio);

        let effects = &mut world.client(ALPHA).effects;
        let direct_db = effects
            .direct_speech_compressor_mut()
            .unwrap()
            .over_threshold_db();
        let radio_db = effects.radio_compressor_mut().unwrap().over_threshold_db();
        assert!(direct_db > 0.0, "the voice is loud enough to compress");
        assert_eq!(
            direct_db,
            direct_reference.over_threshold_db(),
            "frame {index}"
        );
        assert_eq!(
            radio_db,
            radio_reference.over_threshold_db(),
            "frame {index}"
        );
        assert_ne!(direct_db, shared_instance.over_threshold_db());
    }
}

/// A busy scene: `alpha` heard directly through HRTF and over a short-range radio and a speaker
/// radio, `bravo` behind objects in another vehicle.
fn busy_world() -> World {
    let mut world = World::with_hrtf();
    direct_and_radio(&mut world);
    world.add_speaker("speaker_radio", SW_FREQUENCY, at(4.0, 0.0, 0.0), 7.0, 1.0);
    world.client(BRAVO).position = at(-5.0, 2.0, 0.0);
    world.client(BRAVO).object_interception = 2;
    world.ensure(ALPHA);
    world.ensure(BRAVO);
    world
}

#[test]
fn a_thousand_frames_leave_every_scratch_capacity_unchanged() {
    let mut world = busy_world();
    let before = world.processor.scratch_capacities();
    assert!(before.stereo_frames.iter().all(|&frames| frames >= FRAMES));
    assert!(before.routes >= 3 && before.route_plans >= 3);
    let speaker = &world.session.game.speakers_on(SW_FREQUENCY)[0];
    let (speaker_id, speaker_vehicle) =
        (Arc::clone(&speaker.radio_id), Arc::clone(&speaker.vehicle));
    let handles = || {
        (
            Arc::strong_count(&speaker_id),
            Arc::strong_count(&speaker_vehicle),
        )
    };
    let shared_before = handles();
    for index in 0..1000 {
        let input = sine(700.0, 6000.0, index);
        for sender in [ALPHA, BRAVO] {
            world.process(sender, &input);
            assert_eq!(handles(), shared_before, "no reception outlives its frame");
        }
    }
    assert_eq!(world.processor.scratch_capacities(), before);
}

#[test]
fn a_thousand_frames_leave_the_senders_buffers_unchanged() {
    let mut world = busy_world();
    let capacity = |world: &mut World| {
        world
            .client(BRAVO)
            .effects
            .object_interception_buffer_mut()
            .capacity_frames()
    };
    let before = capacity(&mut world);
    for index in 0..1000 {
        world.process(BRAVO, &sine(700.0, 6000.0, index));
    }
    assert_eq!(capacity(&mut world), before);
}

#[test]
fn ensuring_twice_creates_nothing_the_second_time() {
    let mut world = World::with_hrtf();
    direct_and_radio(&mut world);
    world.add_speaker("speaker_radio", SW_FREQUENCY, at(4.0, 0.0, 0.0), 7.0, 1.0);
    let created = world.ensure(ALPHA);
    assert!(created > 0);
    assert_eq!(world.ensure(ALPHA), 0);
    let effects = &mut world.client(ALPHA).effects;
    assert!(effects.personal_radio_mut(LOCAL_RADIO_ID).is_some());
    assert!(effects.speaker_filter_mut("speaker_radio").is_some());
    assert!(effects.hrtf_source_mut("speaker_radio").is_some());
}

#[test]
fn a_sender_out_of_direct_range_gets_no_voice_hrtf_source() {
    let mut world = World::with_hrtf();
    world.client(ALPHA).position = at(0.0, 300.0, 0.0);
    world.ensure(ALPHA);
    let effects = &mut world.client(ALPHA).effects;
    assert!(
        effects
            .hrtf_source_mut(tfar_client_state_1_0::prelude::VOICE_HRTF_KEY)
            .is_none()
    );
    assert!(effects.direct_speech_compressor_mut().is_some());
}

#[test]
fn ensuring_every_sender_covers_the_local_players_speaker_voice() {
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
    let dsp = tfar_client_state_1_0::prelude::DspContext {
        hrir_library: None,
        object_interception_strength: 400.0,
        dropout_seed: 1,
        frames_per_block: FRAMES,
    };
    let created = ensure_every_sender_dsp(
        &mut world.session,
        &world.settings,
        &world.antennas,
        &dsp,
        world.now,
    )
    .unwrap();
    assert!(created > 0);
    let own = world.session.clients.own_mut().unwrap();
    assert!(own.effects.speaker_filter_mut("speaker_radio").is_some());
}
