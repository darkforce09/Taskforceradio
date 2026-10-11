//! Unit tests for `ClientDirectory`.

use super::*;

const OWN: TeamspeakClientId = TeamspeakClientId(1);

/// The original indexes clients by a hash of the nickname and returns the first entry
/// of a colliding hash. Here the full nickname is the key: across many similar names, each
/// lookup finds exactly its own client.
#[test]
fn nicknames_are_keyed_by_their_full_text_so_no_two_share_an_entry() {
    let mut directory = ClientDirectory::new();
    let names: Vec<String> = (0_u16..2000)
        .map(|number| format!("Rifleman {number}"))
        .collect();
    for (number, name) in (10_u16..).zip(&names) {
        directory.client_joined(TeamspeakClientId(number), name);
    }
    assert_eq!(directory.len(), names.len());
    for (number, name) in (10_u16..).zip(&names) {
        assert_eq!(
            directory.get(name).unwrap().client_id(),
            TeamspeakClientId(number)
        );
        assert_eq!(
            directory
                .by_id(TeamspeakClientId(number))
                .unwrap()
                .nickname(),
            name
        );
    }
}

/// A rename moves the entry with its state; the old name finds nothing.
#[test]
fn a_rename_moves_the_entry_with_its_state() {
    let mut directory = ClientDirectory::new();
    directory.client_joined(TeamspeakClientId(5), "old name");
    directory.get_mut("old name").unwrap().voice_volume = 40;
    directory.client_renamed(TeamspeakClientId(5), "new name");
    assert!(directory.get("old name").is_none());
    let client = directory.get("new name").unwrap();
    assert_eq!(client.voice_volume, 40);
    assert_eq!(client.nickname(), "new name");
    assert_eq!(
        directory.by_id(TeamspeakClientId(5)).unwrap().nickname(),
        "new name"
    );
}

#[test]
fn a_join_of_a_known_id_renames_it_and_keeps_its_state() {
    let mut directory = ClientDirectory::new();
    directory.client_joined(TeamspeakClientId(5), "a");
    directory.get_mut("a").unwrap().range = 9;
    directory.client_joined(TeamspeakClientId(5), "b");
    assert_eq!(directory.len(), 1);
    assert_eq!(directory.get("b").unwrap().range, 9);
}

#[test]
fn a_new_client_taking_a_held_nickname_replaces_the_holder() {
    let mut directory = ClientDirectory::new();
    directory.client_joined(TeamspeakClientId(5), "a");
    directory.client_joined(TeamspeakClientId(6), "a");
    assert_eq!(directory.len(), 1);
    assert!(directory.by_id(TeamspeakClientId(5)).is_none());
    assert_eq!(
        directory.get("a").unwrap().client_id(),
        TeamspeakClientId(6)
    );
}

#[test]
fn the_local_player_never_leaves() {
    let mut directory = ClientDirectory::new();
    directory.set_own_client(OWN, "me");
    directory.client_joined(TeamspeakClientId(5), "a");
    directory.client_left(OWN);
    directory.client_left(TeamspeakClientId(5));
    assert!(directory.own().is_some());
    assert!(directory.get("a").is_none());
    directory.client_joined(TeamspeakClientId(6), "b");
    directory.every_client_left();
    assert_eq!(directory.len(), 1);
    assert_eq!(directory.own().unwrap().nickname(), "me");
    assert!(directory.is_own(directory.get("me").unwrap()));
}

#[test]
fn renaming_an_unknown_id_changes_nothing() {
    let mut directory = ClientDirectory::new();
    directory.client_renamed(TeamspeakClientId(5), "a");
    assert!(directory.is_empty());
}
