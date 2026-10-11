//! Unit tests for `GameCommand::parse` dispatch and `counts_as_game_activity`, against
//! `processGameCommand` in `reference/tfar_0_9_12/ts/src/plugin.cpp:1769-1936`.

use super::*;
use crate::error::Error;

#[test]
fn an_empty_message_is_unknown() {
    // `split` of an empty string gives no field, plugin.cpp:1318; nothing matches, FAIL at 1935.
    assert_eq!(
        GameCommand::parse("").unwrap(),
        GameCommand::Unknown(String::new())
    );
}

#[test]
fn an_unknown_command_keeps_the_whole_message() {
    assert_eq!(
        GameCommand::parse("HELLO\tWORLD").unwrap(),
        GameCommand::Unknown("HELLO\tWORLD".to_owned())
    );
}

#[test]
fn command_names_match_case_sensitively() {
    assert_eq!(
        GameCommand::parse("pos\tA").unwrap(),
        GameCommand::Unknown("pos\tA".to_owned())
    );
}

#[test]
fn killed_with_exactly_two_fields_is_parsed() {
    // The original needs `tokens.size() > 2` (plugin.cpp:1775); Arma sends 2
    // (fn_sendPlayerKilled.sqf: `format["KILLED\t%1", name _this]`).
    assert_eq!(
        GameCommand::parse("KILLED\tAlpha 1-1").unwrap(),
        GameCommand::Killed {
            nickname: "Alpha 1-1".to_owned()
        }
    );
}

#[test]
fn killed_without_a_nickname_is_a_field_count_error() {
    assert_eq!(
        GameCommand::parse("KILLED").unwrap_err(),
        Error::WrongFieldCount {
            command: "KILLED",
            expected: 2,
            found: 1
        }
    );
}

#[test]
fn a_trailing_empty_field_is_dropped_before_the_count_like_getline() {
    // `split` follows `std::getline` (plugin.cpp:1318): "TRACK\t" is one field.
    assert!(matches!(
        GameCommand::parse("TRACK\t"),
        Err(Error::WrongFieldCount {
            command: "TRACK",
            expected: 2,
            found: 1
        })
    ));
}

#[test]
fn release_all_tangents_track_and_is_speaking_carry_their_text() {
    // plugin.cpp:1785, 1799 and 1909: each exactly 2 fields.
    assert_eq!(
        GameCommand::parse("RELEASE_ALL_TANGENTS\tAlpha").unwrap(),
        GameCommand::ReleaseAllTangents {
            nickname: "Alpha".to_owned()
        }
    );
    assert_eq!(
        GameCommand::parse("TRACK\tpiwik.php?a=1&b=2").unwrap(),
        GameCommand::Track {
            path: "piwik.php?a=1&b=2".to_owned()
        }
    );
    assert_eq!(
        GameCommand::parse("IS_SPEAKING\tBravo").unwrap(),
        GameCommand::IsSpeaking {
            nickname: "Bravo".to_owned()
        }
    );
    assert!(GameCommand::parse("IS_SPEAKING\tA\tB").is_err());
}

#[test]
fn version_carries_addon_channel_and_password() {
    // plugin.cpp:1789-1797, fn_sendVersionInfo.sqf: `VERSION\t%1\t%2\t%3`.
    assert_eq!(
        GameCommand::parse("VERSION\t0.9.12\tTaskForceRadio\t123").unwrap(),
        GameCommand::Version {
            addon_version: "0.9.12".to_owned(),
            serious_channel_name: "TaskForceRadio".to_owned(),
            serious_channel_password: "123".to_owned(),
        }
    );
}

#[test]
fn version_with_an_empty_password_parses_although_getline_drops_it() {
    // plugin.cpp:1789 needs 4 fields; "VERSION\tv\tname\t" splits into 3 and is refused.
    assert_eq!(
        GameCommand::parse("VERSION\t0.9.12\tTaskForceRadio\t").unwrap(),
        GameCommand::Version {
            addon_version: "0.9.12".to_owned(),
            serious_channel_name: "TaskForceRadio".to_owned(),
            serious_channel_password: String::new(),
        }
    );
    assert!(matches!(
        GameCommand::parse("VERSION\t0.9.12"),
        Err(Error::WrongFieldCount {
            command: "VERSION",
            expected: 4,
            found: 2
        })
    ));
}

#[test]
fn every_command_but_version_counts_as_game_activity() {
    let version = GameCommand::parse("VERSION\ta\tb\tc").unwrap();
    assert!(!version.counts_as_game_activity());
    for message in [
        "TS_INFO\tPING",
        "KILLED\tA",
        "SPEAKERS",
        "RELEASE_ALL_TANGENTS\tA",
        "TRACK\tp",
        "IS_SPEAKING\tA",
        "TANGENT\tPRESSED\t87.6\t5000\tdigital",
        "HELLO",
    ] {
        assert!(
            GameCommand::parse(message)
                .unwrap()
                .counts_as_game_activity(),
            "{message}"
        );
    }
}

#[test]
fn an_unknown_message_starting_with_version_does_not_count_as_game_activity() {
    let command = GameCommand::parse("VERSIONS").unwrap();
    assert_eq!(command, GameCommand::Unknown("VERSIONS".to_owned()));
    assert!(!command.counts_as_game_activity());
}
