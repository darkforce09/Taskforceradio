//! Unit tests for the radio sound choice and shaping.

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

fn constant_block(value: i16) -> StereoBuffer {
    let mut buffer = StereoBuffer::with_frames(480);
    buffer.as_mut_slice().fill(value);
    buffer
}

#[test]
fn the_local_sounds_follow_the_subtype_folder() {
    assert_eq!(
        local_radio_sound(&RadioSubtype::Digital, true).as_deref(),
        Some("radio-sounds/sw/local_start")
    );
    assert_eq!(
        local_radio_sound(&RadioSubtype::DigitalLongRange, false).as_deref(),
        Some("radio-sounds/lr/local_end")
    );
    assert_eq!(
        local_radio_sound(&RadioSubtype::Airborne, true).as_deref(),
        Some("radio-sounds/ab/local_start")
    );
}

#[test]
fn diver_phone_and_unknown_radios_have_no_local_sound() {
    for subtype in [
        RadioSubtype::DiverDigital,
        RadioSubtype::Phone,
        RadioSubtype::Unknown("radio".to_owned()),
    ] {
        assert_eq!(local_radio_sound(&subtype, true), None);
    }
}

#[test]
fn every_named_sound_is_preloaded() {
    for subtype in [
        RadioSubtype::Digital,
        RadioSubtype::DigitalLongRange,
        RadioSubtype::Airborne,
    ] {
        for is_pressed in [true, false] {
            let sound = local_radio_sound(&subtype, is_pressed).unwrap();
            assert!(RADIO_SOUNDS.contains(&sound.as_str()), "{sound}");
        }
    }
}

#[test]
fn a_carried_radio_sound_keeps_only_its_ear() {
    let mut left = constant_block(1000);
    shape_remote_sound(&mut left, &placement(false, StereoMode::Left), None, 20.0);
    assert!(
        left.as_slice()
            .as_chunks::<2>()
            .0
            .iter()
            .all(|frame| frame[0] == 500)
    );
    let mut right = constant_block(1000);
    let listener = Some((
        Position::from_array([0.0; 3]).unwrap(),
        Direction::from_array([0.0, 1.0, 0.0]).unwrap(),
    ));
    shape_remote_sound(
        &mut right,
        &placement(false, StereoMode::Right),
        listener,
        20.0,
    );
    assert!(
        right
            .as_slice()
            .as_chunks::<2>()
            .0
            .iter()
            .all(|frame| frame[0] == 0 && frame[1] == 500)
    );
}

#[test]
fn a_speaker_sound_beyond_its_reach_is_silent() {
    let mut buffer = constant_block(10_000);
    let mut far = placement(true, StereoMode::Both);
    far.position = Some(Position::from_array([5_000.0, 0.0, 0.0]).unwrap());
    let listener = Some((
        Position::from_array([0.0; 3]).unwrap(),
        Direction::from_array([0.0, 1.0, 0.0]).unwrap(),
    ));
    shape_remote_sound(&mut buffer, &far, listener, 20.0);
    assert!(buffer.as_slice().iter().all(|sample| *sample == 0));
}
