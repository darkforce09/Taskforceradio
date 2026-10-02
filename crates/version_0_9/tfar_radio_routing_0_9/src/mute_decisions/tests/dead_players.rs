//! Unit tests for the mute of players without current data, against `setMuteForDeadPlayers` in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:1356`.

use super::*;
use tfar_client_state_0_9::ClientData;

/// `Me` (1) and `Alive` (2) are current; `Dead` (3) was killed; client 4 has no entry.
fn server() -> ServerRadioData {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.current_data_frame = 2;
    server.is_alive = true;
    for (nickname, id) in [("Me", 1), ("Alive", 2), ("Dead", 3)] {
        let mut client = ClientData::new(ClientId(id));
        client.data_frame = 2;
        server.clients.insert(nickname.to_owned(), client);
    }
    server.apply_killed("Dead");
    server
}

const CHANNEL: [ClientId; 4] = [ClientId(1), ClientId(2), ClientId(3), ClientId(4)];

#[test]
fn a_living_listener_in_serious_mode_mutes_every_player_without_current_data() {
    let server = server();
    let mut tracker = MuteTracker::new();
    let decision = set_mute_for_dead_players(&server, &mut tracker, &CHANNEL, ClientId(1), true);
    assert_eq!(decision.mute, [ClientId(3), ClientId(4)]);
    assert_eq!(decision.unmute, Vec::<ClientId>::new());
    assert!(
        set_mute_for_dead_players(&server, &mut tracker, &CHANNEL, ClientId(1), true).is_empty()
    );
}

#[test]
fn outside_serious_mode_or_dead_the_listener_unmutes_only_what_it_muted() {
    let mut server = server();
    let mut tracker = MuteTracker::new();
    set_mute_for_dead_players(&server, &mut tracker, &CHANNEL, ClientId(1), true);
    let decision = set_mute_for_dead_players(&server, &mut tracker, &CHANNEL, ClientId(1), false);
    assert_eq!(decision.unmute, [ClientId(3), ClientId(4)]);
    tracker.decide(ClientId(3), true);
    server.is_alive = false;
    let decision = set_mute_for_dead_players(&server, &mut tracker, &CHANNEL, ClientId(1), true);
    assert_eq!(decision.unmute, [ClientId(3)]);
    assert_eq!(decision.mute, Vec::<ClientId>::new());
}

#[test]
fn the_local_player_is_never_muted() {
    let mut server = server();
    server.client_mut("Me").unwrap().data_frame = 0;
    let mut tracker = MuteTracker::new();
    let decision = set_mute_for_dead_players(&server, &mut tracker, &CHANNEL, ClientId(1), true);
    assert!(!decision.mute.contains(&ClientId(1)));
}
