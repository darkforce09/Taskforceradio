//! `parse`: one game message text to its `GameCommand`.
//!
//! **Role:** split a message at its tabs, find its command by exact name and hand the fields to
//! that command's parser.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::game_commands`; built on `MessageFields` and the command
//! parsers; called by the plugin for every sync call and every queued async message.
//!
//! **Signals and state:** none; pure parsing, callable from any thread.
//!
//! **Invariants:** the split keeps every field, as `helpers::split` does; the text is the message
//! without the trailing `~` of an async call (the game link removes it); an unknown name is
//! `GameCommand::Unknown`, never an error; the token counts are those of the table in the
//! crate README.

use tfar_foundation::arma_values::separators::FIELD;

use crate::game_commands::command_name::CommandName;
use crate::game_commands::commands::{
    TangentKind, TsInfoQuery, parse_config_assignment, parse_frequency_report,
    parse_position_update, parse_speaker_records, parse_tangent_press, parse_tower_records,
    parse_tower_removals, parse_tracking_event,
};
use crate::game_commands::error::Result;
use crate::game_commands::fields::{MessageFields, split_keeping_every_field};
use crate::game_commands::game_command::GameCommand;

#[cfg(test)]
#[path = "tests/parse.rs"]
mod tests;

/// The command in `text`, one game message without the async `~`.
///
/// # Errors
///
/// [`crate::game_commands::Error::WrongTokenCount`] when a known command has a token count it does not accept,
/// and [`crate::game_commands::Error::Malformed`] when one of its fields cannot be read. The message is rejected
/// whole; [`crate::game_commands::Error::sync_reply`] gives the answer to a sync call.
pub fn parse(text: &str) -> Result<GameCommand> {
    let tokens = split_keeping_every_field(text, FIELD);
    let name = tokens.first().copied().unwrap_or_default();
    let Some(command) = CommandName::from_wire_name(name) else {
        return Ok(GameCommand::Unknown {
            name: name.to_owned(),
        });
    };
    let fields = MessageFields::new(command, tokens);
    Ok(match command {
        CommandName::TsInfo => {
            fields.require_at_least(2)?;
            GameCommand::TsInfo(TsInfoQuery::parse(fields.text(1)?))
        }
        CommandName::Position => GameCommand::Position(parse_position_update(&fields)?),
        CommandName::IsSpeaking => {
            fields.require_at_least(2)?;
            GameCommand::IsSpeaking {
                nickname: fields.nickname(1)?,
            }
        }
        CommandName::IsSpeakingBulk => GameCommand::IsSpeakingBulk(
            fields
                .after_name()
                .iter()
                .map(|nickname| tfar_foundation::arma_values::sanitize_nickname(nickname))
                .collect(),
        ),
        CommandName::Frequencies => GameCommand::Frequencies(parse_frequency_report(&fields)?),
        CommandName::Killed => {
            fields.require_count(2)?;
            GameCommand::Killed {
                nickname: fields.nickname(1)?,
            }
        }
        CommandName::Track => GameCommand::Track(parse_tracking_event(&fields)?),
        CommandName::DataFrame => GameCommand::DataFrame,
        CommandName::Speakers => GameCommand::Speakers(parse_speaker_records(&fields)?),
        CommandName::Tangent => {
            GameCommand::Tangent(parse_tangent_press(TangentKind::ShortRange, &fields)?)
        }
        CommandName::TangentLongRange => {
            GameCommand::Tangent(parse_tangent_press(TangentKind::LongRange, &fields)?)
        }
        CommandName::TangentDiverDigital => {
            GameCommand::Tangent(parse_tangent_press(TangentKind::DiverDigital, &fields)?)
        }
        CommandName::ReleaseAllTangents => {
            fields.require_count(2)?;
            GameCommand::ReleaseAllTangents {
                nickname: fields.nickname(1)?,
            }
        }
        CommandName::SetConfig => GameCommand::SetConfig(parse_config_assignment(&fields)?),
        CommandName::MissionEnd => GameCommand::MissionEnd,
        CommandName::RadioTowerAdd => GameCommand::RadioTowerAdd(parse_tower_records(&fields)?),
        CommandName::RadioTowerRemove => {
            GameCommand::RadioTowerRemove(parse_tower_removals(&fields)?)
        }
        CommandName::CollectDebugInfo => GameCommand::CollectDebugInfo,
    })
}
