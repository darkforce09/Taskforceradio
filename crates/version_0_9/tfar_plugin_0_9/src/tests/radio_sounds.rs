//! Unit tests for the radio sound paths.

use super::*;

#[test]
fn each_subtype_plays_from_its_folder_and_a_phone_from_none() {
    assert_eq!(sound_folder(&RadioSubtype::Digital), Some("sw"));
    assert_eq!(sound_folder(&RadioSubtype::DigitalLongRange), Some("lr"));
    assert_eq!(sound_folder(&RadioSubtype::DiverDigital), Some("dd"));
    assert_eq!(sound_folder(&RadioSubtype::Airborne), Some("ab"));
    assert_eq!(sound_folder(&RadioSubtype::Phone), None);
    assert_eq!(
        sound_folder(&RadioSubtype::Unknown("tf_x".to_owned())),
        None
    );
}

#[test]
fn the_paths_are_those_of_the_dist_layout() {
    let sounds = RadioSounds::new(Path::new("/plugins"));
    assert_eq!(
        sounds.game_start(),
        Path::new("/plugins/radio-sounds/on.wav")
    );
    assert_eq!(
        sounds.game_end(),
        Path::new("/plugins/radio-sounds/off.wav")
    );
    assert_eq!(
        sounds.tangent(&RadioSubtype::Airborne, TangentMoment::RemoteEnd),
        Some(PathBuf::from("/plugins/radio-sounds/ab/remote_end.wav"))
    );
    assert_eq!(
        hrir_sphere_path(Path::new("/plugins")),
        Path::new("/plugins/tfar_hrir/IRC_1002_C.bin")
    );
}

#[test]
fn the_preload_lists_every_remote_sound_once() {
    let sounds = RadioSounds::new(Path::new("/plugins"));
    let remote = sounds.remote_sounds();
    assert_eq!(remote.len(), 8);
    assert!(
        remote
            .iter()
            .all(|path| path.to_string_lossy().contains("/remote_"))
    );
    assert_eq!(TangentMoment::ALL.len(), 4);
}
