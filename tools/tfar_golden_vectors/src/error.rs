//! The error type of `tfar_golden_vectors`.

/// Every failure `tfar_golden_vectors` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation asked for is not offered by this crate.
    #[error("tfar_golden_vectors does not offer the operation `{operation}`")]
    UnsupportedOperation {
        /// The name of the operation that was asked for.
        operation: &'static str,
    },
}

/// The result of a fallible `tfar_golden_vectors` operation.
pub type Result<T> = core::result::Result<T, Error>;
