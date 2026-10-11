//! Unit tests for the radio sound list and the own-voice stream wiring.

use tfar_plugin_logic_1_0::client_state::prelude::{
    RadioSubtype, local_radio_sound, remote_radio_sound,
};

use super::*;

#[test]
fn every_sound_the_session_names_is_preloaded() {
    for subtype in [
        RadioSubtype::Digital,
        RadioSubtype::DigitalLongRange,
        RadioSubtype::Airborne,
    ] {
        for is_start in [true, false] {
            let local = local_radio_sound(&subtype, is_start).unwrap();
            let remote = remote_radio_sound(&subtype, is_start).unwrap();
            assert!(RADIO_SOUNDS.contains(&local), "{local}");
            assert!(RADIO_SOUNDS.contains(&remote), "{remote}");
        }
    }
}

#[test]
fn the_own_voice_reaches_the_mixer_stream() {
    let (sounds, problems) = RadioSounds::load(None);
    assert_eq!(problems, Vec::<String>::new());
    assert_eq!(
        sounds.mixer.stream_queued_frames(OWN_RADIO_VOICE_STREAM),
        Some(0)
    );
    assert!(sounds.push_own_voice(&[100_i16; 960]));
    assert_eq!(
        sounds.mixer.stream_queued_frames(OWN_RADIO_VOICE_STREAM),
        Some(480)
    );
    let mut block = [0_i16; 960];
    let mut mask = 0;
    sounds
        .mixer
        .mix_into(&mut block, 2, 480, &mut mask)
        .unwrap();
    assert!(block.iter().all(|sample| *sample == 100));
    assert_eq!(mask & 3, 3);
}

#[test]
fn without_a_plugins_folder_no_sound_is_queued() {
    let (sounds, _) = RadioSounds::load(None);
    sounds.play_local("radio-sounds/sw/local_start", 1.0, StereoMode::Both);
    assert_eq!(sounds.mixer.active_playback_count(), 0);
    assert_eq!(sounds.sound_path("radio-sounds/on"), None);
}
