//! Unit tests for the remote push-to-talk sounds.

use super::*;

/// One second of a constant stereo level.
fn constant_wave(level: i16) -> Vec<i16> {
    vec![level; 48_000 * 2]
}

fn plan(
    listener: Option<ListenerView>,
    is_on_loudspeaker: bool,
    position: Position,
) -> RemoteSoundPlan {
    RemoteSoundPlan {
        path: PathBuf::from("/plugins/radio-sounds/sw/remote_start.wav"),
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
    let playback = render_remote_sound(
        &constant_wave(10_000),
        &plan(None, false, Position::ORIGIN),
        None,
    )
    .unwrap();
    assert!(playback.samples().iter().all(|sample| *sample == 5_000));
}

#[test]
fn without_a_sphere_the_simple_ild_places_the_sound() {
    let playback = render_remote_sound(
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
    let far = Position::from_array([500.0, 0.0, 0.0]).unwrap();
    let playback = render_remote_sound(
        &constant_wave(10_000),
        &plan(Some(listener()), true, far),
        None,
    )
    .unwrap();
    assert!(playback.samples().iter().all(|sample| *sample == 0));
}

#[test]
fn a_ragged_wave_is_refused() {
    assert!(render_remote_sound(&[1, 2, 3], &plan(None, false, Position::ORIGIN), None).is_err());
}
