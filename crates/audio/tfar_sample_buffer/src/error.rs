//! The error type of `tfar_sample_buffer`.

/// Every failure `tfar_sample_buffer` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation asked for is not offered by this crate.
    #[error("tfar_sample_buffer does not offer the operation `{operation}`")]
    UnsupportedOperation {
        /// The name of the operation that was asked for.
        operation: &'static str,
    },
}

/// The result of a fallible `tfar_sample_buffer` operation.
pub type Result<T> = core::result::Result<T, Error>;
