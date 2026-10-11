//! Unit tests for the `MISSIONEND` test of the async worker.

use super::*;

#[test]
fn missionend_alone_or_with_arguments_is_a_mission_end() {
    assert!(is_mission_end(b"MISSIONEND"));
    assert!(is_mission_end(b"MISSIONEND\tsomething"));
}

#[test]
fn other_commands_and_near_names_are_not_a_mission_end() {
    for text in [
        &b"DFRAME"[..],
        b"MISSIONENDED",
        b"MISSION",
        b"missionend",
        b"",
        b"SETCFG\tMISSIONEND",
    ] {
        assert!(!is_mission_end(text), "{text:?}");
    }
}
