//! The error type of `tfar_game_commands_1_0`.

use crate::command_name::CommandName;
use crate::replies::{ASYNC_COMMAND_SENT_IN_SYNC_CONTEXT, FAIL};

/// Every failure `tfar_game_commands_1_0` reports: a known command whose message cannot be read.
/// The message is rejected whole; the plugin logs the error and answers a sync call with
/// [`Error::sync_reply`].
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The message has a token count the command does not accept.
    #[error(
        "{} needs {}, the name included, but has {found}",
        command.wire_name(),
        token_count_text(*minimum, *maximum)
    )]
    WrongTokenCount {
        /// The command the message names.
        command: CommandName,
        /// The fewest tokens accepted, the name included.
        minimum: usize,
        /// The most tokens accepted, the name included; `usize::MAX` for no limit.
        maximum: usize,
        /// The tokens the message has.
        found: usize,
    },
    /// A field of the message cannot be read.
    #[error("{} field `{field}` is malformed: `{value}`", command.wire_name())]
    Malformed {
        /// The command the message names.
        command: CommandName,
        /// The field that failed, by its meaning (`position`, `voice volume`).
        field: &'static str,
        /// The text of the field.
        value: String,
    },
}

impl Error {
    /// The command whose message failed.
    pub const fn command(&self) -> CommandName {
        match self {
            Self::WrongTokenCount { command, .. } | Self::Malformed { command, .. } => *command,
        }
    }

    /// The answer to a sync call whose message failed: `ASYNC COMMAND SENT IN SYNC CONTEXT` for an
    /// async-only command (the original answers so before reading any field), `00` for `POS` and
    /// `IS_SPEAKING` (the not-speaking code), and `FAIL` for `TS_INFO` and the rest.
    pub const fn sync_reply(&self) -> &'static str {
        let command = self.command();
        if command.is_async_only() {
            return ASYNC_COMMAND_SENT_IN_SYNC_CONTEXT;
        }
        match command {
            CommandName::Position | CommandName::IsSpeaking => "00",
            _ => FAIL,
        }
    }
}

/// `exactly 4 tokens`, `at least 2 tokens` or `5 to 6 tokens`.
fn token_count_text(minimum: usize, maximum: usize) -> String {
    if minimum == maximum {
        format!("exactly {minimum} tokens")
    } else if maximum == usize::MAX {
        format!("at least {minimum} tokens")
    } else {
        format!("{minimum} to {maximum} tokens")
    }
}

/// The result of a fallible `tfar_game_commands_1_0` operation.
pub type Result<T> = core::result::Result<T, Error>;
