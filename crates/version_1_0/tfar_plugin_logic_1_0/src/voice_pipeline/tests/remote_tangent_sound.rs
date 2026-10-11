//! Unit tests for the positional remote tangent sound, against the positional `playWavFile` in
//! `reference/tfar_1_0/ts/src/PlaybackHandler.cpp:147-262`.

use super::*;

fn placement(is_on_ground: bool, stereo_mode: StereoMode) -> RemoteSoundPlacement {
    RemoteSoundPlacement {
        gain: 0.5,
        position: None,
        is_on_ground,
        volume: 7.0,
        is_underwater: false,
        vehicle_volume_loss: 0.0,
        is_same_vehicle: true,
        stereo_mode,
    }
}

fn plan(placement: RemoteSoundPlacement, listener: Option<RemoteSoundListener>) -> RemoteSoundPlan {
    RemoteSoundPlan {
        placement,
        listener,
        speaker_distance: 20.0,
    }
}

fn listener() -> RemoteSoundListener {
    RemoteSoundListener {
        position: Position::from_array([0.0; 3]).unwrap(),
        view: Direction::from_array([0.0, 1.0, 0.0]).unwrap(),
    }
}

/// 480 frames of a constant stereo level.
fn constant_wave(value: i16) -> Vec<i16> {
    vec![value; 480 * 2]
}

/// 4800 frames of a 2 kHz square wave, in the speaker band.
fn speaker_band_wave() -> Vec<i16> {
    (0..4_800_usize)
        .flat_map(|frame| {
            let level = if (frame / 12) % 2 == 0 { 8_000 } else { -8_000 };
            [level, level]
        })
        .collect()
}

fn frames(playback: &Playback) -> &[[i16; 2]] {
    playback.samples().as_chunks::<2>().0
}

#[test]
fn without_the_local_player_only_the_gain_applies() {
    let playback = render_remote_tangent_sound(
        &constant_wave(1000),
        &plan(placement(false, StereoMode::Left), None),
    )
    .unwrap();
    assert!(playback.samples().iter().all(|sample| *sample == 500));
}

#[test]
fn a_carried_radio_sound_keeps_only_its_ear() {
    let left = render_remote_tangent_sound(
        &constant_wave(1000),
        &plan(placement(false, StereoMode::Left), Some(listener())),
    )
    .unwrap();
    assert!(frames(&left).iter().all(|frame| *frame == [500, 0]));
    let right = render_remote_tangent_sound(
        &constant_wave(1000),
        &plan(placement(false, StereoMode::Right), Some(listener())),
    )
    .unwrap();
    assert!(frames(&right).iter().all(|frame| *frame == [0, 500]));
}

#[test]
fn a_speaker_sound_beyond_its_reach_is_silent() {
    let mut far = placement(true, StereoMode::Both);
    far.position = Some(Position::from_array([5_000.0, 0.0, 0.0]).unwrap());
    let playback =
        render_remote_tangent_sound(&constant_wave(10_000), &plan(far, Some(listener()))).unwrap();
    assert!(playback.samples().iter().all(|sample| *sample == 0));
}

#[test]
fn a_speaker_sound_is_placed_by_the_simple_ild() {
    let mut beside = placement(true, StereoMode::Both);
    beside.position = Some(Position::from_array([3.0, 0.0, 0.0]).unwrap());
    let playback =
        render_remote_tangent_sound(&speaker_band_wave(), &plan(beside, Some(listener()))).unwrap();
    let energy = |channel: usize| -> f64 {
        frames(&playback)
            .iter()
            .map(|frame| f64::from(frame[channel]).powi(2))
            .sum()
    };
    assert!(
        energy(1) > 2.0 * energy(0),
        "a speaker to the east is on the right"
    );
}

#[test]
fn a_ragged_wave_is_refused() {
    let refused = render_remote_tangent_sound(
        &[1, 2, 3],
        &plan(placement(false, StereoMode::Both), Some(listener())),
    );
    assert!(matches!(refused, Err(Error::RemoteTangentSound { .. })));
}
