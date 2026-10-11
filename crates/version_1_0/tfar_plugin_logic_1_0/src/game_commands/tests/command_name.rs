//! Unit tests for `CommandName`, against `toGameCommand` (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:165-205`).

use super::*;

#[test]
fn every_name_of_the_original_switch_is_known() {
    let names: Vec<&str> = CommandName::ALL
        .iter()
        .map(|name| name.wire_name())
        .collect();
    assert_eq!(
        names,
        [
            "TS_INFO",
            "POS",
            "IS_SPEAKING",
            "IS_SPEAKING_BULK",
            "FREQ",
            "KILLED",
            "TRACK",
            "DFRAME",
            "SPEAKERS",
            "TANGENT",
            "TANGENT_LR",
            "TANGENT_DD",
            "RELEASE_ALL_TANGENTS",
            "SETCFG",
            "MISSIONEND",
            "RadioTwrAdd",
            "RadioTwrDel",
            "collectDebugInfo",
        ]
    );
    for name in CommandName::ALL {
        assert_eq!(CommandName::from_wire_name(name.wire_name()), Some(name));
    }
}

#[test]
fn names_match_exactly() {
    assert_eq!(CommandName::from_wire_name("pos"), None);
    assert_eq!(CommandName::from_wire_name("POS "), None);
    assert_eq!(CommandName::from_wire_name("radiotwradd"), None);
    assert_eq!(CommandName::from_wire_name("PO"), None);
}

#[test]
fn recv_freqs_is_not_a_command() {
    // The original declares RECV_FREQS but toGameCommand never maps it (CommandProcessor.cpp:145).
    assert_eq!(CommandName::from_wire_name("RECV_FREQS"), None);
}

#[test]
fn only_the_four_sync_commands_answer_outside_the_queue() {
    // CommandProcessor.cpp:83-161.
    let sync: Vec<CommandName> = CommandName::ALL
        .into_iter()
        .filter(|name| !name.is_async_only())
        .collect();
    assert_eq!(
        sync,
        [
            CommandName::TsInfo,
            CommandName::Position,
            CommandName::IsSpeaking,
            CommandName::IsSpeakingBulk
        ]
    );
}
