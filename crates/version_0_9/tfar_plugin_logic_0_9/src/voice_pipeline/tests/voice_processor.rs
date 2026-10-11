//! Scenario tests for the gates of `VoiceProcessor::process_sender`, its plugin-enabled cache,
//! its missing sound state, the compressor pass and its allocation, against the stereo function
//! in `reference/tfar_0_9_12/ts/src/plugin.cpp:2653-2828`.

use std::sync::Arc;
use std::time::Duration;

use crate::client_state::ClientId;
use crate::client_state::prelude::{RadioSubtype, TangentKind};
use tfar_audio::dynamics::{Compressor, CompressorSettings};
use tfar_audio::sample_buffer::StereoBuffer;

use super::*;
use crate::voice_pipeline::test_scene::{ALPHA, FRAMES, FREQUENCY, ME, Scene, Signal, at, rms};

const TONE: Signal = Signal::Tone(1000.0, 8000.0);

/// One frame of [`TONE`].
fn tone_frame() -> StereoBuffer {
    let mut frame = StereoBuffer::with_frames(FRAMES);
    TONE.fill(&mut frame, 0, FRAMES);
    frame
}

/// The scene with serious mode on in the channel `Serious`.
fn serious_scene() -> Scene {
    let mut scene = Scene::new();
    scene.state.serious_channel_name = "Serious".to_owned();
    scene.prepare("Alpha");
    scene
}

#[test]
fn a_sender_without_tfar_data_passes_through_untouched() {
    let mut scene = Scene::new();
    let mut frame = tone_frame();
    let context = VoiceFrameContext {
        sender: ClientId(9),
        sender_metadata_shows_plugin: false,
        ..Scene::context(ClientId(9))
    };
    assert_eq!(
        scene.process_with(&context, &mut frame),
        VoiceOutcome::PassThrough
    );
    assert_eq!(frame, tone_frame());
}

#[test]
fn a_sender_whose_metadata_shows_no_plugin_passes_through() {
    let mut scene = Scene::new();
    let mut frame = tone_frame();
    let context = VoiceFrameContext {
        sender_metadata_shows_plugin: false,
        ..Scene::context(ALPHA)
    };
    assert_eq!(
        scene.process_with(&context, &mut frame),
        VoiceOutcome::PassThrough
    );
    assert_eq!(frame, tone_frame());
}

#[test]
fn a_dead_listener_in_serious_mode_hears_silence() {
    let mut scene = serious_scene();
    scene.state.is_alive = false;
    let mut frame = tone_frame();
    let context = VoiceFrameContext {
        sender_channel_name: "Serious",
        ..Scene::context(ALPHA)
    };
    assert_eq!(
        scene.process_with(&context, &mut frame),
        VoiceOutcome::Silenced
    );
    assert!(frame.as_slice().iter().all(|&sample| sample == 0));
}

#[test]
fn a_dead_listener_outside_serious_mode_still_hears_the_chain() {
    let mut scene = serious_scene();
    scene.state.is_alive = false;
    let mut frame = tone_frame();
    assert_eq!(
        scene.process_with(&Scene::context(ALPHA), &mut frame),
        VoiceOutcome::Processed
    );
    assert!(rms(frame.as_slice(), 0) > 100.0);
}

#[test]
fn an_alive_listener_in_serious_mode_hears_a_tfar_player_without_current_data_as_silence() {
    let mut scene = serious_scene();
    scene.client("Alpha").data_frame = 1;
    let mut frame = tone_frame();
    let context = VoiceFrameContext {
        sender_channel_name: "Serious",
        ..Scene::context(ALPHA)
    };
    assert_eq!(
        scene.process_with(&context, &mut frame),
        VoiceOutcome::Silenced
    );
    let outside_game = VoiceFrameContext {
        is_in_game: false,
        ..context
    };
    let mut frame = tone_frame();
    assert_eq!(
        scene.process_with(&outside_game, &mut frame),
        VoiceOutcome::PassThrough
    );
}

#[test]
fn a_serious_channel_named_like_another_radio_plugin_channel_passes_through() {
    let mut scene = Scene::new();
    scene.state.serious_channel_name = "RT".to_owned();
    scene.client("Alpha").data_frame = 1;
    let mut frame = tone_frame();
    let context = VoiceFrameContext {
        sender_channel_name: "RT",
        ..Scene::context(ALPHA)
    };
    assert_eq!(
        scene.process_with(&context, &mut frame),
        VoiceOutcome::PassThrough
    );
}

#[test]
fn the_local_players_own_frame_without_current_data_is_silence() {
    let mut scene = Scene::new();
    scene.client("Me").data_frame = 1;
    let mut frame = tone_frame();
    assert_eq!(
        scene.process_with(&Scene::context(ME), &mut frame),
        VoiceOutcome::Silenced
    );
    assert!(frame.as_slice().iter().all(|&sample| sample == 0));
}

#[test]
fn a_positive_plugin_check_stands_in_for_failed_ones_for_ten_seconds() {
    let mut scene = Scene::new();
    scene.prepare("Alpha");
    let mut frame = tone_frame();
    assert_eq!(
        scene.process_with(&Scene::context(ALPHA), &mut frame),
        VoiceOutcome::Processed
    );
    let no_metadata = VoiceFrameContext {
        sender_metadata_shows_plugin: false,
        ..Scene::context(ALPHA)
    };
    scene.now += Duration::from_secs(9);
    let mut frame = tone_frame();
    assert_eq!(
        scene.process_with(&no_metadata, &mut frame),
        VoiceOutcome::Processed
    );
    scene.now += Duration::from_secs(2);
    let mut frame = tone_frame();
    assert_eq!(
        scene.process_with(&no_metadata, &mut frame),
        VoiceOutcome::PassThrough
    );
}

#[test]
fn a_frame_missing_a_needed_filter_silences_that_path_and_is_counted() {
    let mut scene = Scene::new();
    scene.client("Alpha").can_speak = false;
    let mut frame = tone_frame();
    assert_eq!(
        scene.process_with(&Scene::context(ALPHA), &mut frame),
        VoiceOutcome::Processed
    );
    assert!(frame.as_slice().iter().all(|&sample| sample == 0));
    assert_eq!(scene.processor.frames_missing_sound_state(), 1);
    scene.prepare("Alpha");
    let mut frame = tone_frame();
    scene.process_with(&Scene::context(ALPHA), &mut frame);
    assert_eq!(scene.processor.frames_missing_sound_state(), 1);
}

#[test]
fn the_global_volume_is_the_last_gain() {
    let mut full = Scene::new();
    full.prepare("Alpha");
    let mut half = Scene::new();
    half.state.global_volume = 0.5;
    half.prepare("Alpha");
    let full_level = rms(&full.run(ALPHA, &TONE, 10), 0);
    let half_level = rms(&half.run(ALPHA, &TONE, 10), 0);
    assert!(
        (half_level / full_level - 0.5).abs() < 0.01,
        "{half_level} / {full_level}"
    );
}

/// A sender heard over two receptions: the listener's own radio and a loudspeaker.
fn two_reception_scene() -> Scene {
    let mut scene = Scene::with_hrtf();
    scene.client("Alpha").position = at(0.0, 100.0, 0.0);
    scene.transmit(
        "Alpha",
        TangentKind::ShortRange,
        RadioSubtype::Digital,
        1000,
    );
    scene.add_loudspeaker(at(5.0, 0.0, 0.0), 10, 1.0);
    scene.prepare("Alpha");
    scene
}

#[test]
fn the_compressor_envelope_advances_once_per_frame_whatever_the_receptions() {
    let mut scene = two_reception_scene();
    let mut reference = Compressor::new(CompressorSettings::VERSION_0_9).unwrap();
    let signal = Signal::Tone(700.0, 20_000.0);
    let mut frame = StereoBuffer::with_frames(FRAMES);
    for index in 0..20 {
        signal.fill(&mut frame, index * FRAMES, FRAMES);
        let mut reference_frame = frame.clone();
        reference.process_stereo_i16(&mut reference_frame);
        scene.process_with(&Scene::context(ALPHA), &mut frame);
        let compressor = scene.client("Alpha").dsp.compressor_mut().unwrap();
        assert_eq!(
            compressor.over_threshold_db().to_bits(),
            reference.over_threshold_db().to_bits(),
            "frame {index}"
        );
    }
    assert!(reference.over_threshold_db() > 1.0);
}

#[test]
fn a_thousand_frames_change_no_scratch_capacity() {
    let mut scene = two_reception_scene();
    scene.client("Alpha").position = at(3.0, 4.0, 0.0);
    scene.prepare("Alpha");
    let before = scene.processor.scratch().capacities();
    let speaker_id = Arc::clone(&scene.state.speakers_on(FREQUENCY)[0].radio_id);
    let shared_handles = Arc::strong_count(&speaker_id);
    let mut frame = StereoBuffer::with_frames(4 * FRAMES);
    let signal = Signal::Noise(6000.0);
    let mut start = 0;
    for index in 0..1000 {
        let frames = [FRAMES, 960, 441, 1][index % 4];
        signal.fill(&mut frame, start, frames);
        start += frames;
        assert_eq!(
            scene.process_with(&Scene::context(ALPHA), &mut frame),
            VoiceOutcome::Processed
        );
        let scratch = scene.processor.scratch();
        assert!(scratch.heard.is_empty() && scratch.routes.is_empty());
        assert_eq!(Arc::strong_count(&speaker_id), shared_handles);
    }
    assert_eq!(scene.processor.scratch().capacities(), before);
    assert_eq!(scene.processor.frames_missing_sound_state(), 0);
}
