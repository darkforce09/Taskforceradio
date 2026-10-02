//! Unit tests for `FREQ`, against the `FREQ` branch in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:1874-1908`.

use super::*;
use tfar_game_commands_0_9::GameCommand;
use tfar_game_commands_0_9::prelude::StereoMode;

fn update(nickname: &str) -> FrequencyUpdate {
    let message = format!(
        "FREQ\t[\"30.1|7|1\",\"30.1|5|2\",\"31|3|0\"]\t[\"50|9|0\"]\t40\ttrue\t20\t6.7\t{nickname}\t0.25\t8\t0.5\t99\t1.5\t25"
    );
    match GameCommand::parse(&message).unwrap() {
        GameCommand::Frequencies(update) => update,
        other => panic!("not a FREQ: {other:?}"),
    }
}

#[test]
fn every_field_is_stored_and_a_repeated_frequency_keeps_its_last_setting() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.apply_frequencies(&update("Me"), "Me");
    assert_eq!(
        server.sw_frequencies.get("30.1"),
        Some(&FrequencySettings {
            volume: 5,
            stereo_mode: StereoMode::Right,
        })
    );
    assert_eq!(server.sw_frequencies.get("31").unwrap().volume, 3);
    assert_eq!(server.lr_frequencies.get("50").unwrap().volume, 9);
    assert_eq!(server.dd_frequency, "40");
    assert!(server.is_alive);
    assert_eq!(server.my_voice_volume, 20);
    assert_eq!(server.dd_volume_level, 6);
    assert!((server.waves_level - 0.25).abs() < 1e-12);
    assert!((server.terrain_intersection_coefficient - 8.0).abs() < 1e-12);
    assert!((server.global_volume - 0.5).abs() < 1e-12);
    assert!((server.receiving_distance_multiplier - 1.5).abs() < 1e-12);
    assert!((server.speaker_distance - 25.0).abs() < 1e-12);
}

#[test]
fn a_new_list_replaces_the_old_one() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.apply_frequencies(&update("Me"), "Me");
    let empty =
        match GameCommand::parse("FREQ\t[]\t[]\t\ttrue\t20\t6\tMe\t0\t7\t1\t20\t1\t20").unwrap() {
            GameCommand::Frequencies(update) => update,
            other => panic!("not a FREQ: {other:?}"),
        };
    server.apply_frequencies(&empty, "Me");
    assert!(server.sw_frequencies.is_empty());
    assert!(server.lr_frequencies.is_empty());
}

#[test]
fn a_different_game_nickname_asks_for_a_rename() {
    let mut server = ServerRadioData::new("Sam".to_owned());
    assert_eq!(
        server.apply_frequencies(&update("Sgt Pepper"), "Sam"),
        Some(RenameRequest {
            current_nickname: "Sam".to_owned(),
            game_nickname: "Sgt Pepper".to_owned(),
        })
    );
}

#[test]
fn no_rename_for_the_same_name_an_unknown_current_name_or_a_placeholder() {
    let mut server = ServerRadioData::new("Sam".to_owned());
    assert_eq!(server.apply_frequencies(&update("Sam"), "Sam"), None);
    assert_eq!(server.apply_frequencies(&update("Sgt Pepper"), ""), None);
    for placeholder in NICKNAMES_NEVER_ADOPTED {
        assert_eq!(
            server.apply_frequencies(&update(placeholder), "Sam"),
            None,
            "{placeholder}"
        );
    }
}

#[test]
fn the_original_nickname_after_two_renames() {
    // The original stores the current name on every rename, so after a second rename it would
    // restore the first game name at game end.
    let mut server = ServerRadioData::new("Sam".to_owned());
    let first = server
        .apply_frequencies(&update("Sgt Pepper"), "Sam")
        .unwrap();
    server.confirm_rename(&first);
    let second = server
        .apply_frequencies(&update("Cpl Lennon"), "Sgt Pepper")
        .unwrap();
    server.confirm_rename(&second);
    assert_eq!(server.my_original_nickname.as_deref(), Some("Sam"));
    assert_eq!(server.end_game().restore_nickname.as_deref(), Some("Sam"));
}

#[test]
fn a_refused_rename_saves_no_original_nickname() {
    let mut server = ServerRadioData::new("Sam".to_owned());
    let _refused = server.apply_frequencies(&update("Sgt Pepper"), "Sam");
    assert_eq!(server.my_original_nickname, None);
    assert_eq!(server.end_game().restore_nickname, None);
}
