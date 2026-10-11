//! The error type of `tfar_plugin_logic_0_9::radio_routing`.

/// Every failure `tfar_plugin_logic_0_9::radio_routing` reports: the reasons a sender's voice needs no sound
/// state.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    /// No entry carries the sender's nickname.
    #[error("no client entry for the sender `{nickname}`")]
    UnknownSender {
        /// The sender's nickname.
        nickname: String,
    },
    /// The sender's data frame is not current: they are dead, or their data is stale.
    #[error("the data of the sender `{nickname}` is not current")]
    SenderDataNotCurrent {
        /// The sender's nickname.
        nickname: String,
    },
    /// The local player has no entry with a current data frame.
    #[error("the listener (client {client_id}) has no current data")]
    ListenerDataNotCurrent {
        /// The listener's TeamSpeak client id.
        client_id: u16,
    },
}

/// The result of a fallible `tfar_plugin_logic_0_9::radio_routing` operation.
pub type Result<T> = core::result::Result<T, Error>;
