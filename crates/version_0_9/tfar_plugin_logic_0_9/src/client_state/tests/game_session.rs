//! Unit tests for `GameSession` and the game start and end, against the pipe thread
//! (`reference/tfar_0_9_12/ts/src/plugin.cpp:2056-2071`), the service thread (plugin.cpp:1992-2009)
//! and `onGameEnd` (plugin.cpp:1391).

use super::*;

#[test]
fn in_game_without_a_serious_channel() {
    // The original starts a game only when the mission names a serious channel.
    let start = Instant::now();
    let mut server = ServerRadioData::new("Me".to_owned());
    server.apply_version("1.0", "", "");
    let mut session = GameSession::new(start);
    assert!(
        !session.record_message(false, start),
        "VERSION never starts a game"
    );
    assert!(!session.is_in_game());
    assert!(
        session.record_message(true, start),
        "the first POS starts it"
    );
    assert!(session.is_in_game());
    assert_eq!(
        server.start_game(),
        GameStart {
            serious_channel: None
        }
    );
    assert_eq!(server.current_data_frame, 0);
}

#[test]
fn a_serious_channel_adds_the_channel_move_to_the_start() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.apply_version("1.0", "TFAR serious", "secret");
    assert_eq!(
        server.start_game().serious_channel,
        Some(SeriousChannel {
            name: "TFAR serious".to_owned(),
            password: "secret".to_owned(),
        })
    );
}

#[test]
fn only_the_first_activity_of_a_game_starts_it() {
    let start = Instant::now();
    let mut session = GameSession::new(start);
    assert!(session.record_message(true, start));
    assert!(!session.record_message(true, start + Duration::from_secs(1)));
    assert_eq!(session.last_game_activity(), start + Duration::from_secs(1));
}

#[test]
fn a_version_message_does_not_stamp_activity() {
    let start = Instant::now();
    let mut session = GameSession::new(start);
    session.record_message(true, start);
    session.record_message(false, start + Duration::from_secs(3));
    assert_eq!(session.last_game_activity(), start);
    assert!(session.game_timed_out(start + Duration::from_millis(4001)));
}

#[test]
fn the_game_times_out_after_more_than_four_seconds_without_activity() {
    let start = Instant::now();
    let mut session = GameSession::new(start);
    session.record_message(true, start);
    assert!(!session.game_timed_out(start + GAME_ACTIVITY_TIMEOUT));
    assert_eq!(session.check_timeout(start + GAME_ACTIVITY_TIMEOUT), None);
    let late = start + GAME_ACTIVITY_TIMEOUT + Duration::from_millis(1);
    assert_eq!(
        session.check_timeout(late),
        Some(GameTimeout { ended_a_game: true })
    );
    assert!(!session.is_in_game());
    assert_eq!(session.last_game_activity(), late);
    assert!(
        session.record_message(true, late),
        "the next activity starts a new game"
    );
}

#[test]
fn a_timeout_outside_a_game_ends_nothing_and_restamps() {
    let start = Instant::now();
    let mut session = GameSession::new(start);
    let late = start + Duration::from_secs(5);
    assert_eq!(
        session.check_timeout(late),
        Some(GameTimeout {
            ended_a_game: false
        })
    );
    assert!(!session.game_timed_out(late + Duration::from_secs(4)));
}

#[test]
fn the_end_of_a_game_kills_the_listener_and_forgets_the_serious_channel() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.apply_version("1.0", "TFAR serious", "secret");
    server.start_game();
    server.is_alive = true;
    server.my_original_nickname = Some("Sam".to_owned());
    let end = server.end_game();
    assert_eq!(end.restore_nickname.as_deref(), Some("Sam"));
    assert!(!server.is_alive);
    assert_eq!(server.current_data_frame, INVALID_DATA_FRAME);
    assert_eq!(server.serious_channel_name, "");
    assert_eq!(server.serious_channel_password, "");
    assert_eq!(server.my_original_nickname, None);
    assert_eq!(server.end_game().restore_nickname, None, "handed back once");
}
