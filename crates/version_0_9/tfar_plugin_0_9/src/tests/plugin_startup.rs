//! Unit tests for the plugin start-up resources.

use std::time::Duration;

use super::*;
use crate::fake_team_speak::{FakeAction, FakeTeamSpeak, missing_plugin_folder};

#[test]
fn a_missing_sphere_turns_the_hrtf_off_with_a_chat_line() {
    let fake = FakeTeamSpeak::with_players(&[]);
    assert!(load_hrir_sphere(&missing_plugin_folder(), &*fake).is_none());
    let actions = fake.take_actions();
    assert_eq!(actions.len(), 1);
    assert!(matches!(&actions[0], FakeAction::Print(text) if text.contains("IRC_1002_C.bin")));
}

#[test]
fn without_a_settings_database_a_release_is_immediate() {
    assert_eq!(push_to_talk_delay(None), Duration::ZERO);
    assert_eq!(
        push_to_talk_delay(Some(&missing_plugin_folder())),
        Duration::ZERO
    );
}
