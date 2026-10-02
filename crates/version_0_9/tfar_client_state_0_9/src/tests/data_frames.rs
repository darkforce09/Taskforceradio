//! Unit tests for the data frames and expiry, against `hasClientData` and
//! `removeExpiredPositions` in `reference/tfar_0_9_12/ts/src/plugin.cpp:584, 1938`.

use super::*;
use crate::server_radio_data::INVALID_DATA_FRAME;

fn server_at_frame(frame: i32) -> ServerRadioData {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.current_data_frame = frame;
    server
}

fn insert(
    server: &mut ServerRadioData,
    nickname: &str,
    id: u16,
    frame: i32,
    time: Option<Instant>,
) {
    let mut client = ClientData::new(ClientId(id));
    client.data_frame = frame;
    client.position_time = time;
    server.clients.insert(nickname.to_owned(), client);
}

#[test]
fn a_frame_is_current_within_one_of_the_current_frame() {
    let server = server_at_frame(10);
    assert!(server.is_frame_current(9));
    assert!(server.is_frame_current(10));
    assert!(server.is_frame_current(11));
    assert!(!server.is_frame_current(8));
    assert!(!server.is_frame_current(12));
    assert!(!server.is_frame_current(INVALID_DATA_FRAME));
}

#[test]
fn fresh_data_needs_a_matching_client_id_and_a_current_frame() {
    let mut server = server_at_frame(5);
    insert(&mut server, "Alpha", 2, 4, None);
    insert(&mut server, "Bravo", 3, 2, None);
    assert!(server.has_fresh_data(ClientId(2)));
    assert!(!server.has_fresh_data(ClientId(3)));
    assert!(!server.has_fresh_data(ClientId(4)));
}

#[test]
fn a_lookup_by_client_id_returns_the_fresh_entry_not_a_stale_one_left_by_a_rename() {
    // Client 2 was "Alpha" and is now "Bravo": the old entry keeps the id until it expires and
    // sorts first, where the original's getClientData would return it.
    let now = Instant::now();
    let mut server = server_at_frame(5);
    insert(&mut server, "Alpha", 2, 1, Some(now));
    insert(&mut server, "Bravo", 2, 5, Some(now));
    let (nickname, _) = server.fresh_client_by_id(ClientId(2)).unwrap();
    assert_eq!(nickname, "Bravo");
    assert_eq!(server.fresh_nickname_of(ClientId(2)), Some("Bravo"));
    server.fresh_client_by_id_mut(ClientId(2)).unwrap().range = 4;
    assert_eq!(server.client("Bravo").unwrap().range, 4);
    assert_eq!(server.client("Alpha").unwrap().range, 0);
}

#[test]
fn of_two_current_entries_with_one_id_the_last_positioned_wins() {
    let now = Instant::now();
    let mut server = server_at_frame(5);
    insert(&mut server, "Alpha", 2, 5, Some(now));
    insert(
        &mut server,
        "Bravo",
        2,
        5,
        Some(now + Duration::from_secs(1)),
    );
    assert_eq!(server.fresh_nickname_of(ClientId(2)), Some("Bravo"));
    server.client_mut("Alpha").unwrap().position_time = Some(now + Duration::from_secs(2));
    assert_eq!(server.fresh_nickname_of(ClientId(2)), Some("Alpha"));
}

#[test]
fn an_entry_expires_after_twenty_seconds_without_a_position() {
    let start = Instant::now();
    let mut server = server_at_frame(5);
    insert(&mut server, "Alpha", 2, 5, Some(start));
    assert_eq!(
        server.remove_expired(start + POSITION_EXPIRY),
        Vec::<String>::new()
    );
    let removed = server.remove_expired(start + POSITION_EXPIRY + Duration::from_millis(1));
    assert_eq!(removed, vec!["Alpha".to_owned()]);
    assert!(server.clients.is_empty());
}

#[test]
fn an_entry_expires_when_its_frame_is_more_than_one_away() {
    let now = Instant::now();
    let mut server = server_at_frame(5);
    insert(&mut server, "Alpha", 2, 4, Some(now));
    insert(&mut server, "Bravo", 3, 3, Some(now));
    insert(&mut server, "Charlie", 4, 7, Some(now));
    assert_eq!(
        server.remove_expired(now),
        vec!["Bravo".to_owned(), "Charlie".to_owned()]
    );
    assert!(server.client("Alpha").is_some());
}

#[test]
fn an_entry_never_positioned_expires_at_the_next_sweep() {
    let mut server = server_at_frame(0);
    insert(&mut server, "Alpha", 2, 0, None);
    assert_eq!(
        server.remove_expired(Instant::now()),
        vec!["Alpha".to_owned()]
    );
}
