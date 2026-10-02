//! The error type of `tfar_plugin_commands_1_0`.

/// Every failure `tfar_plugin_commands_1_0` reports: a plugin command that cannot be decoded.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// No command has this name and token count. The message is the original's log line
    /// (`reference/tfar_1_0/ts/src/plugin.cpp:1066`).
    #[error("UNKNOWN PLUGIN COMMAND {text}")]
    Unrecognised {
        /// The command text as received.
        text: String,
    },
    /// A field of a known command cannot be read.
    #[error("{command} field `{field}` is malformed: `{value}`")]
    Malformed {
        /// The command's name.
        command: &'static str,
        /// The field that failed, by its meaning.
        field: &'static str,
        /// The text of the field.
        value: String,
    },
}

/// The result of a fallible `tfar_plugin_commands_1_0` operation.
pub type Result<T> = core::result::Result<T, Error>;
