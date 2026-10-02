//! The messages whose fields are plain text: `KILLED`, `RELEASE_ALL_TANGENTS`, `VERSION`,
//! `TRACK` and `IS_SPEAKING`.

use crate::command_names::{IS_SPEAKING, KILLED, RELEASE_ALL_TANGENTS, TRACK, VERSION};
use crate::error::{Error, Result};
use crate::field_values::exact_fields;
use crate::game_command::GameCommand;

#[cfg(test)]
#[path = "tests/text_field_messages.rs"]
mod tests;

/// `KILLED\t<nickname>`: 2 fields or more. The game sends exactly 2; fields after the nickname
/// are ignored, as the original ignores them.
pub(crate) fn parse_killed(fields: &[&str]) -> Result<GameCommand> {
    match fields {
        [_, nickname, ..] => Ok(GameCommand::Killed {
            nickname: (*nickname).to_owned(),
        }),
        _ => Err(Error::WrongFieldCount {
            command: KILLED,
            expected: 2,
            found: fields.len(),
        }),
    }
}

/// `RELEASE_ALL_TANGENTS\t<nickname>`: exactly 2 fields.
pub(crate) fn parse_release_all_tangents(fields: &[&str]) -> Result<GameCommand> {
    let [_, nickname] = exact_fields::<2>(RELEASE_ALL_TANGENTS, fields)?;
    Ok(GameCommand::ReleaseAllTangents {
        nickname: nickname.to_owned(),
    })
}

/// `VERSION\t<addon version>\t<serious channel name>\t<serious channel password>`: exactly 4
/// fields. A field may be empty but the last: `std::getline` drops a trailing empty field, so a
/// mission without a serious channel password sends 3 fields and is refused, as the original
/// refuses it.
pub(crate) fn parse_version(fields: &[&str]) -> Result<GameCommand> {
    let [
        _,
        addon_version,
        serious_channel_name,
        serious_channel_password,
    ] = exact_fields::<4>(VERSION, fields)?;
    Ok(GameCommand::Version {
        addon_version: addon_version.to_owned(),
        serious_channel_name: serious_channel_name.to_owned(),
        serious_channel_password: serious_channel_password.to_owned(),
    })
}

/// `TRACK\t<path>`: exactly 2 fields.
pub(crate) fn parse_track(fields: &[&str]) -> Result<GameCommand> {
    let [_, path] = exact_fields::<2>(TRACK, fields)?;
    Ok(GameCommand::Track {
        path: path.to_owned(),
    })
}

/// `IS_SPEAKING\t<nickname>`: exactly 2 fields.
pub(crate) fn parse_is_speaking(fields: &[&str]) -> Result<GameCommand> {
    let [_, nickname] = exact_fields::<2>(IS_SPEAKING, fields)?;
    Ok(GameCommand::IsSpeaking {
        nickname: nickname.to_owned(),
    })
}
