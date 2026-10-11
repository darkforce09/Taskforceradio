//! Unit tests for `message_counts_as_game_activity`, against the pipe thread's
//! `startWith("VERSION", command)` (`reference/tfar_0_9_12/ts/src/plugin.cpp:2056`) and `startWith`
//! (plugin.cpp:494-500).

use super::*;

#[test]
fn a_version_message_does_not_count_as_game_activity() {
    assert!(!message_counts_as_game_activity(
        "VERSION\t0.9.12\tTaskForceRadio\t123"
    ));
    assert!(!message_counts_as_game_activity("VERSIONX"));
}

#[test]
fn every_other_message_counts_as_game_activity() {
    assert!(message_counts_as_game_activity("POS\tA"));
    assert!(message_counts_as_game_activity("TS_INFO\tPING"));
    assert!(message_counts_as_game_activity(""));
}

#[test]
fn a_bare_version_counts_because_start_with_needs_a_longer_text() {
    assert!(message_counts_as_game_activity("VERSION"));
}
