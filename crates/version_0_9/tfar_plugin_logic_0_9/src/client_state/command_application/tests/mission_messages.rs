//! Unit tests for `VERSION`, `SPEAKERS` and `KILLED`, against
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:1562, 1728, 1789`.

use super::*;
use crate::client_state::client_data::ClientData;
use crate::client_state::client_id::ClientId;
use crate::game_commands::GameCommand;

fn records(message: &str) -> Vec<SpeakerRecord> {
    match GameCommand::parse(message).unwrap() {
        GameCommand::Speakers(records) => records,
        other => panic!("not a SPEAKERS message: {other:?}"),
    }
}

#[test]
fn version_stores_the_handshake_and_moves_to_the_next_frame() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.current_data_frame = 3;
    server.apply_version("0.9.12", "Serious", "pw");
    assert_eq!(server.addon_version, "0.9.12");
    assert_eq!(server.serious_channel_name, "Serious");
    assert_eq!(server.serious_channel_password, "pw");
    assert_eq!(server.current_data_frame, 4);
}

#[test]
fn version_outside_a_game_moves_past_the_invalid_frame() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.apply_version("0.9.12", "", "");
    assert_eq!(server.current_data_frame, INVALID_DATA_FRAME + 1);
    server.current_data_frame = i32::MAX;
    server.apply_version("0.9.12", "", "");
    assert_eq!(
        server.current_data_frame,
        i32::MAX,
        "the frame saturates, never wraps"
    );
}

#[test]
fn speakers_are_filed_under_every_frequency_in_the_games_order() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.apply_speakers(&records(
        "SPEAKERS\t\u{b}r1\n30|31\nAlpha\n[]\n5\nno\u{b}r2\n30\nBravo\n[1,0,0]\n8\nno\n-2",
    ));
    let on_30: Vec<_> = server
        .speakers_on("30")
        .iter()
        .map(|s| &*s.radio_id)
        .collect();
    assert_eq!(on_30, ["r1", "r2"]);
    assert_eq!(server.speakers_on("31").len(), 1);
    assert_eq!(server.speakers_on("30")[0].position, None);
    assert!((server.speakers_on("30")[1].wave_z + 2.0).abs() < 1e-12);
}

#[test]
fn an_empty_speaker_list_clears_the_speakers() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.apply_speakers(&records("SPEAKERS\t\u{b}r1\n30\nAlpha\n[]\n5\nno"));
    server.apply_speakers(&[]);
    assert!(server.speakers.is_empty());
}

#[test]
fn killed_invalidates_the_players_frame() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.current_data_frame = 2;
    let mut alpha = ClientData::new(ClientId(2));
    alpha.data_frame = 2;
    server.clients.insert("Alpha".to_owned(), alpha);
    assert!(server.has_fresh_data(ClientId(2)));
    assert!(server.apply_killed("Alpha"));
    assert_eq!(
        server.client("Alpha").unwrap().data_frame,
        INVALID_DATA_FRAME
    );
    assert!(!server.has_fresh_data(ClientId(2)));
    assert!(!server.apply_killed("Nobody"));
}
