//! Unit tests for the game message apply functions.

use super::*;
use crate::client_state::test_support::{
    ALPHA, OWN, at, frequency_report, now, position_update, session,
};
use crate::game_commands::VehicleDescriptor;

#[test]
fn a_position_updates_the_client_and_stamps_frame_and_time() {
    let start = now();
    let mut session = session();
    session.next_data_frame();
    let applied = session
        .apply_position(&position_update("alpha", at(5.0, 6.0, 7.0)), start)
        .unwrap();
    assert_eq!(
        applied,
        PositionApplied {
            client_id: ALPHA,
            is_own_client: false
        }
    );
    let alpha = session.clients.get("alpha").unwrap();
    assert_eq!(alpha.position, at(5.0, 6.0, 7.0));
    assert_eq!(alpha.data_frame, DataFrame::new(1));
    assert_eq!(alpha.position_updated_at, Some(start));
    assert_eq!(alpha.last_activity_at, Some(start));
}

#[test]
fn radio_abilities_come_only_from_the_local_players_own_position() {
    let start = now();
    let mut session = session();
    session.apply_position(&position_update("alpha", at(0.0, 0.0, 0.0)), start);
    let alpha = session.clients.get("alpha").unwrap();
    assert!(!alpha.can_use_sw_radio && !alpha.can_use_lr_radio);
    let applied = session
        .apply_position(&position_update("me", at(0.0, 0.0, 0.0)), start)
        .unwrap();
    assert!(applied.is_own_client);
    let own = session.clients.own().unwrap();
    assert!(own.can_use_sw_radio && own.can_use_lr_radio && !own.can_use_dd_radio);
}

#[test]
fn a_position_of_an_unknown_nickname_changes_nothing() {
    let mut session = session();
    assert!(
        session
            .apply_position(&position_update("ghost", at(0.0, 0.0, 0.0)), now())
            .is_none()
    );
}

#[test]
fn frequencies_ask_for_a_rename_to_a_new_game_name_only() {
    let mut session = session();
    let rename = session
        .apply_frequencies(&frequency_report("Sgt. Rock"))
        .unwrap();
    assert_eq!(rename.client_id, OWN);
    assert_eq!(rename.nickname, "Sgt. Rock");
    assert!(session.game.is_alive);
    assert!(session.apply_frequencies(&frequency_report("me")).is_none());
    for name in crate::game_commands::NICKNAMES_NEVER_ADOPTED {
        assert!(session.apply_frequencies(&frequency_report(name)).is_none());
    }
}

#[test]
fn frequencies_without_a_local_player_set_the_game_data_only() {
    let mut session = SessionState::new();
    assert!(session.apply_frequencies(&frequency_report("x")).is_none());
    assert_eq!(session.game.voice_volume, 20);
}

#[test]
fn a_kill_invalidates_the_client_and_the_local_player_dies_with_its_own() {
    let start = now();
    let mut session = session();
    session.apply_position(&position_update("me", at(0.0, 0.0, 0.0)), start);
    session.apply_position(&position_update("alpha", at(0.0, 0.0, 0.0)), start);
    let applied = session.apply_killed("alpha");
    assert_eq!(applied.killed, Some(ALPHA));
    assert!(applied.is_player_alive);
    assert_eq!(
        session.clients.get("alpha").unwrap().data_frame,
        DataFrame::INVALID
    );
    let applied = session.apply_killed("me");
    assert!(!applied.is_player_alive);
    assert!(!session.game.is_alive);
    assert_eq!(session.apply_killed("ghost").killed, None);
}

#[test]
fn speakers_are_listed_under_each_frequency_with_their_owner() {
    let mut session = session();
    session.apply_speakers(&[
        SpeakerRecord {
            radio_id: "2:390".to_owned(),
            frequencies: vec!["40".to_owned(), "41".to_owned()],
            nickname: "alpha".to_owned(),
            position: None,
            volume: 7,
            vehicle: VehicleDescriptor::not_in_vehicle(),
            wave_z: 1.0,
        },
        SpeakerRecord {
            radio_id: "tf_anprc152_3".to_owned(),
            frequencies: vec!["40".to_owned()],
            nickname: "ghost".to_owned(),
            position: Some(at(1.0, 2.0, 3.0)),
            volume: 5,
            vehicle: VehicleDescriptor::not_in_vehicle(),
            wave_z: 3.0,
        },
    ]);
    let on_40 = session.game.speakers_on("40");
    assert_eq!(on_40.len(), 2);
    assert_eq!(on_40.first().unwrap().owner, Some(ALPHA));
    assert_eq!(on_40.get(1).unwrap().owner, None);
    assert_eq!(session.game.speakers_on("41").len(), 1);
    session.apply_speakers(&[]);
    assert!(session.game.speakers.is_empty());
}
