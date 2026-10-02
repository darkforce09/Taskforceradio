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

/// `VERSION\t<addon version>\t<serious channel name>\t<serious channel password>`: 4 fields, or
/// 3 for an empty password. The game writes all four, but `std::getline` drops the trailing empty
/// field of a mission without a password, so 3 fields mean that password is empty.
pub(crate) fn parse_version(fields: &[&str]) -> Result<GameCommand> {
    let (addon_version, serious_channel_name, serious_channel_password) = match fields {
        [_, addon, name, password] => (addon, name, *password),
        [_, addon, name] => (addon, name, ""),
        _ => {
            return Err(Error::WrongFieldCount {
                command: VERSION,
                expected: 4,
                found: fields.len(),
            });
        }
    };
    Ok(GameCommand::Version {
        addon_version: (*addon_version).to_owned(),
        serious_channel_name: (*serious_channel_name).to_owned(),
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
