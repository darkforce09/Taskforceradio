//! Unit tests for the channel client list, against `updateNicknamesList` in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:1478`.

use super::*;

#[test]
fn an_unknown_nickname_gets_a_fresh_entry_with_its_id() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.update_nicknames(&[
        (ClientId(1), "Me".to_owned()),
        (ClientId(2), "Alpha".to_owned()),
    ]);
    assert_eq!(server.client_id_of("Me"), Some(ClientId(1)));
    assert_eq!(server.client_id_of("Alpha"), Some(ClientId(2)));
    assert_eq!(server.client("Alpha").unwrap().data_frame, 0);
}

#[test]
fn a_known_nickname_keeps_its_state_and_takes_the_new_id() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.update_nicknames(&[(ClientId(2), "Alpha".to_owned())]);
    server.client_mut("Alpha").unwrap().range = 5000;
    server.update_nicknames(&[(ClientId(9), "Alpha".to_owned())]);
    let alpha = server.client("Alpha").unwrap();
    assert_eq!(alpha.client_id, ClientId(9));
    assert_eq!(alpha.range, 5000);
}

#[test]
fn an_entry_missing_from_the_list_stays_until_it_expires() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.update_nicknames(&[(ClientId(2), "Alpha".to_owned())]);
    server.update_nicknames(&[(ClientId(3), "Bravo".to_owned())]);
    assert!(server.client("Alpha").is_some());
    assert!(server.client("Bravo").is_some());
}

#[test]
fn the_local_nickname_is_recorded() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.set_my_nickname("Renamed");
    assert_eq!(server.my_nickname, "Renamed");
}
