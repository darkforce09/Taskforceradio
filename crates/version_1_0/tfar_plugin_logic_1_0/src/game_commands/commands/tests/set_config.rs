//! Unit tests for the `SETCFG` parser, against `CommandProcessor.cpp:388-414`
//! (`reference/tfar_1_0/ts/src/`) and `fnc_setPluginSetting.sqf:28`.

use crate::plugin_settings::{PluginSettings, TypeTag};

use crate::game_commands::error::Error;
use crate::game_commands::{CommandName, ConfigAssignment, GameCommand, parse};

fn assignment(text: &str) -> ConfigAssignment {
    match parse(text).unwrap() {
        GameCommand::SetConfig(assignment) => assignment,
        other => panic!("not SETCFG: {other:?}"),
    }
}

#[test]
fn four_tokens_carry_the_type_tag() {
    let assignment = assignment("SETCFG\tminimumPluginVersion\t339\tSCALAR");
    assert_eq!(assignment.key, "minimumPluginVersion");
    assert_eq!(assignment.value, "339");
    assert_eq!(assignment.type_tag, Some(TypeTag::Scalar));
}

#[test]
fn an_empty_string_value_keeps_its_place() {
    let assignment = assignment("SETCFG\tserious_channelPassword\t\tSTRING");
    assert_eq!(assignment.value, "");
    assert_eq!(assignment.type_tag, Some(TypeTag::Text));
}

#[test]
fn any_other_count_of_three_or_more_tokens_has_no_tag() {
    // tokens.size() != 4 -> the empty-text fallback, CommandProcessor.cpp:411-413.
    assert_eq!(
        assignment("SETCFG\tserious_channelName\tRadio").type_tag,
        None
    );
    assert_eq!(
        assignment("SETCFG\tserious_channelName\tRadio\tSTRING\textra").type_tag,
        None
    );
}

#[test]
fn fewer_than_three_tokens_is_an_error() {
    // The original reads tokens[2] before any count check (CommandProcessor.cpp:390).
    assert_eq!(
        parse("SETCFG\tvoiceCone"),
        Err(Error::WrongTokenCount {
            command: CommandName::SetConfig,
            minimum: 3,
            maximum: usize::MAX,
            found: 2
        })
    );
}

#[test]
fn an_assignment_applies_to_the_settings() {
    let assignment = assignment("SETCFG\tserious_channelName\tTaskForceRadio\tSTRING");
    let mut settings = PluginSettings::default();
    settings
        .set(&assignment.key, &assignment.value, assignment.type_tag)
        .unwrap();
    assert_eq!(settings.serious_channel_name(), "TaskForceRadio");
}

#[test]
fn a_sync_setcfg_is_answered_as_async_only() {
    let error = parse("SETCFG").unwrap_err();
    assert_eq!(error.sync_reply(), "ASYNC COMMAND SENT IN SYNC CONTEXT");
}
