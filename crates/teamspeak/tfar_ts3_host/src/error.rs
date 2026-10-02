//! The error type of `tfar_ts3_host`.

/// Every failure `tfar_ts3_host` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation asked for is not offered by this crate.
    #[error("tfar_ts3_host does not offer the operation `{operation}`")]
    UnsupportedOperation {
        /// The name of the operation that was asked for.
        operation: &'static str,
    },
}

/// The result of a fallible `tfar_ts3_host` operation.
pub type Result<T> = core::result::Result<T, Error>;
