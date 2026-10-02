//! Unit tests for `GameCommand`: names, the async-only rule and the fixed sync answers.

use crate::{CommandName, GameCommand, TsInfoQuery, parse};

#[test]
fn every_parsed_command_reports_its_name() {
    let table = [
        ("TS_INFO\tPING", CommandName::TsInfo),
        ("IS_SPEAKING\tnick", CommandName::IsSpeaking),
        ("IS_SPEAKING_BULK\ta", CommandName::IsSpeakingBulk),
        ("KILLED\tnick", CommandName::Killed),
        ("DFRAME", CommandName::DataFrame),
        ("SPEAKERS\t", CommandName::Speakers),
        ("TANGENT\tPRESSED\t1\t1\tdigital", CommandName::Tangent),
        (
            "TANGENT_LR\tPRESSED\t1\t1\tdigital_lr",
            CommandName::TangentLongRange,
        ),
        (
            "TANGENT_DD\tPRESSED\t1\t1\tdd",
            CommandName::TangentDiverDigital,
        ),
        (
            "RELEASE_ALL_TANGENTS\tnick",
            CommandName::ReleaseAllTangents,
        ),
        ("SETCFG\tvoiceCone\ttrue\tBOOL", CommandName::SetConfig),
        ("MISSIONEND", CommandName::MissionEnd),
        ("RadioTwrAdd\t", CommandName::RadioTowerAdd),
        ("RadioTwrDel\t", CommandName::RadioTowerRemove),
        ("TRACK\ta\tb\t[]", CommandName::Track),
        ("collectDebugInfo", CommandName::CollectDebugInfo),
    ];
    for (text, name) in table {
        assert_eq!(parse(text).unwrap().name(), Some(name), "{text:?}");
    }
    assert_eq!(parse("NOPE").unwrap().name(), None);
}

#[test]
fn the_fixed_sync_answers_are_the_originals() {
    // CommandProcessor.cpp:80, :161, :625, :628.
    assert_eq!(
        parse("RECV_FREQS").unwrap().fixed_sync_reply(),
        Some("UNKNOWN COMMAND")
    );
    assert_eq!(
        parse("DFRAME").unwrap().fixed_sync_reply(),
        Some("ASYNC COMMAND SENT IN SYNC CONTEXT")
    );
    assert_eq!(
        parse("TS_INFO\tPING").unwrap().fixed_sync_reply(),
        Some("PONG")
    );
    assert_eq!(
        parse("TS_INFO\tWHAT").unwrap().fixed_sync_reply(),
        Some("FAIL")
    );
    assert_eq!(parse("TS_INFO\tSERVER").unwrap().fixed_sync_reply(), None);
    assert_eq!(parse("IS_SPEAKING\tnick").unwrap().fixed_sync_reply(), None);
}

#[test]
fn async_only_follows_the_command_name() {
    assert!(parse("DFRAME").unwrap().is_async_only());
    assert!(!parse("TS_INFO\tPING").unwrap().is_async_only());
    assert!(!parse("NOPE").unwrap().is_async_only());
    assert_eq!(
        parse("TS_INFO\tSERVERUID").unwrap(),
        GameCommand::TsInfo(TsInfoQuery::ServerUid)
    );
}
