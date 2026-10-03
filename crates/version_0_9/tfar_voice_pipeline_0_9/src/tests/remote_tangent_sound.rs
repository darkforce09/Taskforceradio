//! Unit tests for the positional remote push-to-talk sound, against `playWavFile` in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:711-785`.

use super::*;
use crate::test_scene::{Signal, at, channel_rms, decibels, real_library};

/// One second of a constant stereo level.
fn constant_wave(level: i16) -> Vec<i16> {
    vec![level; 48_000 * 2]
}

/// One second of a 1 kHz stereo tone at `level`.
fn tone_wave(level: f64) -> Vec<i16> {
    let mut wave = StereoBuffer::with_frames(48_000);
    Signal::Tone(1_000.0, level).fill(&mut wave, 0, 48_000);
    wave.as_slice().to_vec()
}

fn plan(
    listener: Option<ListenerView>,
    is_on_loudspeaker: bool,
    position: Position,
) -> RemoteSoundPlan {
    RemoteSoundPlan {
        gain: 0.5,
        position,
        is_on_loudspeaker,
        radio_volume: 7,
        is_under_water: false,
        speaker_distance: 20.0,
        listener,
    }
}

fn listener() -> ListenerView {
    ListenerView {
        position: Position::ORIGIN,
        view_angle: Angle::ZERO,
        vehicle_isolation: None,
    }
}

#[test]
fn without_listener_data_only_the_gain_applies() {
    let playback = render_remote_tangent_sound(
        &constant_wave(10_000),
        &plan(None, false, Position::ORIGIN),
        None,
    )
    .unwrap();
    assert!(playback.samples().iter().all(|sample| *sample == 5_000));
}

#[test]
fn without_a_sphere_the_simple_ild_places_the_sound() {
    let playback = render_remote_tangent_sound(
        &constant_wave(10_000),
        &plan(Some(listener()), false, Position::ORIGIN),
        None,
    )
    .unwrap();
    let (left, right) = (playback.samples()[0], playback.samples()[1]);
    assert_eq!((left, right), (1_250, 5_000));
}

#[test]
fn a_loudspeaker_beyond_its_reach_is_silent() {
    let playback = render_remote_tangent_sound(
        &constant_wave(10_000),
        &plan(Some(listener()), true, at(500.0, 0.0, 0.0)),
        None,
    )
    .unwrap();
    assert!(playback.samples().iter().all(|sample| *sample == 0));
}

#[test]
fn a_ragged_wave_is_refused() {
    let refused =
        render_remote_tangent_sound(&[1, 2, 3], &plan(None, false, Position::ORIGIN), None);
    assert!(matches!(refused, Err(Error::RemoteTangentSound { .. })));
}

#[test]
fn with_a_sphere_a_radio_to_the_right_is_louder_in_the_right_ear_from_the_first_frame() {
    let library = real_library();
    let playback = render_remote_tangent_sound(
        &tone_wave(8_000.0),
        &plan(Some(listener()), false, at(10.0, 0.0, 0.0)),
        Some(&library),
    )
    .unwrap();
    let samples = playback.samples();
    assert_eq!(samples.len(), 48_000 * 2, "the sound keeps its length");
    let (left, right) = channel_rms(samples, 960);
    assert!(
        decibels(right, left) > 6.0,
        "right over left {} dB",
        decibels(right, left)
    );
    let (onset_left, onset_right) = channel_rms(samples.get(..960).unwrap(), 0);
    assert!(
        onset_left.max(onset_right) > 100.0,
        "the latency is cut off"
    );
}

#[test]
fn a_vehicle_between_listener_and_radio_muffles_the_sound() {
    let open = render_remote_tangent_sound(
        &tone_wave(8_000.0),
        &plan(Some(listener()), false, at(0.0, 2.0, 0.0)),
        None,
    )
    .unwrap();
    let mut muffled_listener = listener();
    muffled_listener.vehicle_isolation = Some(0.8);
    let muffled = render_remote_tangent_sound(
        &tone_wave(8_000.0),
        &plan(Some(muffled_listener), false, at(0.0, 2.0, 0.0)),
        None,
    )
    .unwrap();
    let level = |samples: &[i16]| {
        let (left, right) = channel_rms(samples, 960);
        left + right
    };
    assert!(decibels(level(muffled.samples()), level(open.samples())) < -10.0);
}
