//! Unit tests for `parse`: the dispatch, the unknown names and the simple commands.

use super::*;
use crate::error::Error;

#[test]
fn an_unknown_name_is_unknown_not_an_error() {
    assert_eq!(
        parse("HELLO\tworld"),
        Ok(GameCommand::Unknown {
            name: "HELLO".to_owned()
        })
    );
    assert_eq!(
        parse(""),
        Ok(GameCommand::Unknown {
            name: String::new()
        })
    );
}

#[test]
fn ts_info_reads_only_its_second_field() {
    // The sync split stops after two tokens (CommandProcessor.cpp:78), so extras are ignored.
    assert_eq!(
        parse("TS_INFO\tCHANNELID\textra"),
        Ok(GameCommand::TsInfo(TsInfoQuery::ChannelId))
    );
}

#[test]
fn ts_info_without_a_query_answers_fail() {
    let error = parse("TS_INFO").unwrap_err();
    assert_eq!(error.command(), CommandName::TsInfo);
    assert_eq!(error.sync_reply(), "FAIL");
}

#[test]
fn is_speaking_sanitizes_its_nickname() {
    assert_eq!(
        parse("IS_SPEAKING\t Alpha "),
        Ok(GameCommand::IsSpeaking {
            nickname: "_Alpha_".to_owned()
        })
    );
    assert_eq!(parse("IS_SPEAKING").unwrap_err().sync_reply(), "00");
}

#[test]
fn is_speaking_bulk_lists_every_nickname_including_a_trailing_empty_one() {
    // fnc_processPlayerPositions.sqf:60 joins the names with tabs; the original's split keeps a
    // trailing empty field, which gets its own "00\t".
    assert_eq!(
        parse("IS_SPEAKING_BULK\tAlpha\t Bravo"),
        Ok(GameCommand::IsSpeakingBulk(vec![
            "Alpha".to_owned(),
            "_Bravo".to_owned()
        ]))
    );
    assert_eq!(
        parse("IS_SPEAKING_BULK\tAlpha\t"),
        Ok(GameCommand::IsSpeakingBulk(vec![
            "Alpha".to_owned(),
            String::new()
        ]))
    );
    assert_eq!(
        parse("IS_SPEAKING_BULK"),
        Ok(GameCommand::IsSpeakingBulk(Vec::new()))
    );
}

#[test]
fn killed_and_release_all_tangents_carry_one_nickname() {
    assert_eq!(
        parse("KILLED\tAlpha"),
        Ok(GameCommand::Killed {
            nickname: "Alpha".to_owned()
        })
    );
    assert_eq!(
        parse("RELEASE_ALL_TANGENTS\tAlpha "),
        Ok(GameCommand::ReleaseAllTangents {
            nickname: "Alpha_".to_owned()
        })
    );
    assert!(matches!(
        parse("KILLED"),
        Err(Error::WrongTokenCount {
            command: CommandName::Killed,
            ..
        })
    ));
}

#[test]
fn the_field_free_commands_take_any_token_count() {
    assert_eq!(parse("DFRAME"), Ok(GameCommand::DataFrame));
    assert_eq!(parse("MISSIONEND\tx"), Ok(GameCommand::MissionEnd));
    assert_eq!(parse("collectDebugInfo"), Ok(GameCommand::CollectDebugInfo));
}
