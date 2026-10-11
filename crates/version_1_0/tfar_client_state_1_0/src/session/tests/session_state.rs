//! Unit tests for `SessionState`.

use std::time::Duration;

use super::*;
use crate::test_support::{ALPHA, BRAVO, at, now, position_update, session};

#[test]
fn the_speaking_code_reports_talking_or_transmitting_and_receiving() {
    let mut session = session();
    assert_eq!(&session.speaking_code("nobody"), b"00");
    assert_eq!(&session.speaking_code("alpha"), b"00");
    session.clients.get_mut("alpha").unwrap().is_talking_now = true;
    assert_eq!(&session.speaking_code("alpha"), b"10");
    let own = session.clients.own_mut().unwrap();
    own.receiving.start(ALPHA, "100");
    own.transmitting_over = Some(crate::TangentRadio::ShortRange);
    assert_eq!(&session.speaking_code("me"), b"11");
}

/// Expiry happens in the sweep, which invalidates stale clients and names them.
#[test]
fn the_sweep_expires_stale_clients_and_names_them() {
    let start = now();
    let mut session = session();
    session.apply_position(&position_update("alpha", at(1.0, 0.0, 0.0)), start);
    session.apply_position(&position_update("bravo", at(2.0, 0.0, 0.0)), start);
    session.next_data_frame();
    session.next_data_frame();
    session.apply_position(&position_update("bravo", at(2.0, 0.0, 0.0)), start);
    assert!(!session.is_client_alive(ALPHA, start));
    assert_eq!(
        session.clients.get("alpha").unwrap().data_frame,
        DataFrame::FIRST
    );
    assert_eq!(session.sweep_expired(start), vec![ALPHA]);
    assert_eq!(
        session.clients.get("alpha").unwrap().data_frame,
        DataFrame::INVALID
    );
    assert!(session.is_client_alive(BRAVO, start));
    let later = start + crate::ALIVE_TIMEOUT + Duration::from_secs(1);
    assert_eq!(session.sweep_expired(later), vec![BRAVO]);
}

#[test]
fn the_plugin_check_of_an_unknown_client_is_no() {
    let start = now();
    let mut session = session();
    assert!(!session.plugin_enabled_for(TeamspeakClientId(99), Some("x"), start));
    assert!(!session.needs_plugin_metadata(TeamspeakClientId(99), start));
    assert!(session.needs_plugin_metadata(ALPHA, start));
    session
        .clients
        .get_mut("alpha")
        .unwrap()
        .mark_plugin_enabled(start);
    assert!(!session.needs_plugin_metadata(ALPHA, start));
    assert!(session.plugin_enabled_for(ALPHA, None, start));
}

#[test]
fn a_game_end_marks_the_player_dead_and_the_frame_invalid() {
    let mut session = session();
    session.end_game();
    assert!(!session.game.is_alive);
    assert_eq!(session.game.current_data_frame, DataFrame::INVALID);
}
