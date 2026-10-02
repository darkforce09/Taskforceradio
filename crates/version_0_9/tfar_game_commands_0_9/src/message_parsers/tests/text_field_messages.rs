//! Unit tests for the plain-text messages, against `reference/tfar_0_9_12/ts/src/plugin.cpp:1775`
//! (`KILLED`), 1785 (`RELEASE_ALL_TANGENTS`), 1789 (`VERSION`), 1799 (`TRACK`) and 1909
//! (`IS_SPEAKING`).

use super::*;

#[test]
fn killed_accepts_two_fields_and_ignores_extra_ones() {
    // F-020: Arma sends exactly 2; the original's `> 2` rule accepted only more.
    assert_eq!(
        parse_killed(&["KILLED", "Alpha"]).unwrap(),
        GameCommand::Killed {
            nickname: "Alpha".to_owned()
        }
    );
    assert_eq!(
        parse_killed(&["KILLED", "Alpha", "extra"]).unwrap(),
        GameCommand::Killed {
            nickname: "Alpha".to_owned()
        }
    );
}

#[test]
fn version_fields_may_be_empty_except_the_last() {
    assert_eq!(
        parse_version(&["VERSION", "0.9.12", "", "pw"]).unwrap(),
        GameCommand::Version {
            addon_version: "0.9.12".to_owned(),
            serious_channel_name: String::new(),
            serious_channel_password: "pw".to_owned(),
        }
    );
}

#[test]
fn version_with_three_fields_has_an_empty_password() {
    assert_eq!(
        parse_version(&["VERSION", "0.9.12", ""]).unwrap(),
        GameCommand::Version {
            addon_version: "0.9.12".to_owned(),
            serious_channel_name: String::new(),
            serious_channel_password: String::new(),
        }
    );
    assert!(parse_version(&["VERSION", "0.9.12"]).is_err());
}

#[test]
fn each_two_field_message_refuses_another_count() {
    assert!(parse_release_all_tangents(&["RELEASE_ALL_TANGENTS"]).is_err());
    assert!(parse_release_all_tangents(&["RELEASE_ALL_TANGENTS", "a", "b"]).is_err());
    assert!(parse_track(&["TRACK", "a", "b"]).is_err());
    assert!(parse_is_speaking(&["IS_SPEAKING"]).is_err());
    assert!(parse_version(&["VERSION", "a", "b", "c", "d"]).is_err());
}
