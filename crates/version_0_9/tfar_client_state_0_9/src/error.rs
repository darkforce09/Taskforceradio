//! The error type of `tfar_client_state_0_9`.

/// Every failure `tfar_client_state_0_9` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation asked for is not offered by this crate.
    #[error("tfar_client_state_0_9 does not offer the operation `{operation}`")]
    UnsupportedOperation {
        /// The name of the operation that was asked for.
        operation: &'static str,
    },
}

/// The result of a fallible `tfar_client_state_0_9` operation.
pub type Result<T> = core::result::Result<T, Error>;
