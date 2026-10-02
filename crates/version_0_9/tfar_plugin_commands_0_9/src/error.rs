//! The error type of `tfar_plugin_commands_0_9`.

/// Every failure `tfar_plugin_commands_0_9` reports. The original logs each of these and drops
/// the command (`UNKNOWN PLUGIN COMMAND`), so the plugin does the same.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    /// The command name is none of `TANGENT`, `TANGENT_LR`, `TANGENT_DD`,
    /// `RELEASE_ALL_TANGENTS` and `VOLUME`, or a known name arrived with another field count.
    #[error("`{command}` is not a 0.9.12 plugin command with {found} fields")]
    UnknownCommand {
        /// The whole command text as received.
        command: String,
        /// The field count of the command, split as `std::getline` splits it.
        found: usize,
    },
    /// A field of a known command does not hold the value its position needs.
    #[error("field `{field}` of `{command}` is malformed: {source}")]
    Malformed {
        /// The command name, the first field.
        command: &'static str,
        /// The name of the field at fault.
        field: &'static str,
        /// What the value parser found wrong.
        #[source]
        source: tfar_arma_values::Error,
    },
}

/// The result of a fallible `tfar_plugin_commands_0_9` operation.
pub type Result<T> = core::result::Result<T, Error>;
