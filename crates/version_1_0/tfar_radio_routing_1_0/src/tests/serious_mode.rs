//! Unit tests for the serious channel lookup.

use super::*;

const CHANNELS: [(ChannelId, &str); 3] = [
    (ChannelId(1), "Lobby"),
    (ChannelId(7), "TaskForceRadio"),
    (ChannelId(9), "TaskForceRadio"),
];

#[test]
fn an_empty_name_names_no_channel() {
    assert_eq!(serious_mode_channel("", CHANNELS), None);
}

#[test]
fn an_id_prefix_names_the_channel_by_number() {
    assert_eq!(serious_mode_channel("ID:42", CHANNELS), Some(ChannelId(42)));
    assert_eq!(serious_mode_channel("ID:7", []), Some(ChannelId(7)));
    assert_eq!(serious_mode_channel("ID:x", CHANNELS), None);
    assert_eq!(serious_mode_channel("ID:-3", CHANNELS), None);
}

#[test]
fn a_name_matches_exactly_and_the_last_listed_wins() {
    assert_eq!(
        serious_mode_channel("TaskForceRadio", CHANNELS),
        Some(ChannelId(9))
    );
    assert_eq!(serious_mode_channel("taskforceradio", CHANNELS), None);
    assert_eq!(serious_mode_channel("Lobby ", CHANNELS), None);
}
