//! The error type of `tfar_plugin_logic_0_9::game_commands`.

use crate::game_commands::replies::FAIL;

/// Every failure `tfar_plugin_logic_0_9::game_commands` reports. Each variant names the command and, where one
/// is at fault, the field, so a log line shows what the game sent.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    /// A known command arrived with a field count its parser does not accept.
    #[error("`{command}` needs {expected} fields, the message holds {found}")]
    WrongFieldCount {
        /// The command name, the first field of the message.
        command: &'static str,
        /// The field count the command needs, its name included; KILLED accepts this many or more.
        expected: usize,
        /// The field count of the message, split as `std::getline` splits it.
        found: usize,
    },
    /// A field of a known command does not hold the value its position needs.
    #[error("field `{field}` of `{command}` is malformed: {source}")]
    Malformed {
        /// The command name, the first field of the message.
        command: &'static str,
        /// The name of the field at fault.
        field: &'static str,
        /// What the value parser found wrong.
        #[source]
        source: tfar_foundation::arma_values::Error,
    },
    /// A `SPEAKERS` record holds fewer than the six parts every record carries.
    #[error("speaker record {record} holds {parts} parts where at least 6 are needed")]
    ShortSpeakerRecord {
        /// The zero-based position of the record among the vertical-tab separated records.
        record: usize,
        /// The number of line-feed separated parts the record holds.
        parts: usize,
    },
}

impl Error {
    /// The reply the plugin gives the game for a message that fails to parse: `FAIL`, the reply the
    /// original gives every message it does not handle.
    pub const fn reply(&self) -> &'static str {
        FAIL
    }
}

/// The result of a fallible `tfar_plugin_logic_0_9::game_commands` operation.
pub type Result<T> = core::result::Result<T, Error>;
