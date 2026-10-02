//! The error type of `tfar_web_requests`.

/// Every failure `tfar_web_requests` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation asked for is not offered by this crate.
    #[error("tfar_web_requests does not offer the operation `{operation}`")]
    UnsupportedOperation {
        /// The name of the operation that was asked for.
        operation: &'static str,
    },
}

/// The result of a fallible `tfar_web_requests` operation.
pub type Result<T> = core::result::Result<T, Error>;
