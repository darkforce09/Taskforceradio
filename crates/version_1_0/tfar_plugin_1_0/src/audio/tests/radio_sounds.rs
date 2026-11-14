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
            assert!(MIXED_SOUNDS.contains(&local), "{local}");
            assert!(MIXED_SOUNDS.contains(&remote), "{remote}");
        }
    }
}

/// A stereo 16-bit WAV of 480 silent frames at `rate` Hz.
fn wave_bytes(rate: u32) -> Vec<u8> {
    let data_bytes: u32 = 480 * 4;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&rate.to_le_bytes());
    bytes.extend_from_slice(&(rate * 4).to_le_bytes());
    bytes.extend_from_slice(&4_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    bytes.resize(bytes.len() + data_bytes as usize, 0);
    bytes
}

#[test]
fn the_upstream_44_1_khz_game_sounds_are_left_to_teamspeak_and_refuse_nothing() {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/tmp")
        .join(format!("tfar-1-0-radio-sounds-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    for sound in MIXED_SOUNDS {
        let path = wave_path(&folder, sound);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, wave_bytes(48_000)).unwrap();
    }
    for sound in ["radio-sounds/on", "radio-sounds/off"] {
        std::fs::write(wave_path(&folder, sound), wave_bytes(44_100)).unwrap();
    }
    let waves = WaveLibrary::new();
    let refused = preload_mixed_sounds(&waves, &folder);
    assert!(refused.is_empty(), "{refused:?}");
    assert_eq!(waves.len(), MIXED_SOUNDS.len());
    let _ = std::fs::remove_dir_all(folder);
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
