//! Unit tests for the per-connection state.

use tfar_plugin_logic_0_9::voice_pipeline::DEFAULT_MAX_FRAMES;

use super::*;
use crate::channel_snapshot::read_channel_snapshot;
use crate::fake_team_speak::{FakeTeamSpeak, SERVER};

#[test]
fn a_snapshot_names_the_local_player_and_lists_the_channel() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let mut state = ServerState::new(String::new());
    state.apply_channel_snapshot(&read_channel_snapshot(&*fake, SERVER));
    assert_eq!(state.radio.my_nickname, "Mock Player");
    assert!(state.radio.client("Alpha").is_some());
    assert_eq!(state.my_client(), Some(StateClientId::new(1)));
}

#[test]
fn the_scratch_holds_a_full_block_without_growing() {
    let state = ServerState::new("Me".to_owned());
    assert!(state.voice_scratch.capacity_frames() >= DEFAULT_MAX_FRAMES);
    assert!(state.captured_scratch.capacity_frames() >= DEFAULT_MAX_FRAMES);
}

#[test]
fn sound_state_for_a_sender_without_current_data_is_skipped_quietly() {
    let fake = FakeTeamSpeak::with_players(&["Alpha"]);
    let mut state = ServerState::new(String::new());
    state.apply_channel_snapshot(&read_channel_snapshot(&*fake, SERVER));
    state.ensure_every_sender_dsp(None);
    state.ensure_sender_dsp("Nobody", None);
    assert!(state.radio.client("Alpha").is_some());
}
