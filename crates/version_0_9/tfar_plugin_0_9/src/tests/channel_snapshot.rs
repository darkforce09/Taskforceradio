//! Unit tests for the channel snapshot and the voice facts.

use super::*;
use crate::fake_team_speak::{FakeTeamSpeak, SERIOUS, SERVER};

#[test]
fn the_snapshot_reads_our_channel_with_each_members_plugin_state() {
    let fake = FakeTeamSpeak::with_players(&["Alpha", "Bravo"]);
    fake.world().clients.get_mut(&ClientId(3)).unwrap().channel = SERIOUS;
    let snapshot = read_channel_snapshot(&*fake, SERVER);
    assert_eq!(snapshot.my_client, Some(ClientId(1)));
    assert_eq!(snapshot.my_nickname, "Mock Player");
    assert_eq!(snapshot.channel_name, "Lobby");
    assert_eq!(
        snapshot.nickname_pairs(),
        vec![
            (StateClientId::new(1), "Mock Player".to_owned()),
            (StateClientId::new(2), "Alpha".to_owned()),
        ]
    );
    let shows: Vec<bool> = snapshot
        .members
        .iter()
        .map(|member| member.metadata_shows_plugin)
        .collect();
    assert_eq!(shows, vec![false, true]);
}

#[test]
fn a_disconnected_server_gives_an_empty_snapshot() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    fake.world().is_connected = false;
    assert_eq!(
        read_channel_snapshot(&*fake, SERVER),
        ChannelSnapshot::default()
    );
}

#[test]
fn the_facts_answer_for_known_clients_and_stay_neutral_for_others() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let mut facts = TeamSpeakFacts::default();
    facts.replace_with(&read_channel_snapshot(&*fake, SERVER));
    let alpha = StateClientId::new(2);
    assert_eq!(facts.my_client, Some(StateClientId::new(1)));
    assert!(facts.metadata_shows_plugin(alpha));
    assert_eq!(facts.channel_name_of(alpha), "Lobby");
    let stranger = StateClientId::new(99);
    assert!(!facts.metadata_shows_plugin(stranger));
    assert_eq!(facts.channel_name_of(stranger), "");
    facts.forget(alpha);
    assert!(facts.client(alpha).is_none());
}

#[test]
fn client_ids_convert_both_ways() {
    assert_eq!(host_client_id(state_client_id(ClientId(7))), ClientId(7));
}
