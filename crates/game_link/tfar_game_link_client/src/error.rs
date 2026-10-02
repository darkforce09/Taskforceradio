//! The error type of `tfar_game_link_client`.

/// Every failure `tfar_game_link_client` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation asked for is not offered by this crate.
    #[error("tfar_game_link_client does not offer the operation `{operation}`")]
    UnsupportedOperation {
        /// The name of the operation that was asked for.
        operation: &'static str,
    },
}

/// The result of a fallible `tfar_game_link_client` operation.
pub type Result<T> = core::result::Result<T, Error>;
