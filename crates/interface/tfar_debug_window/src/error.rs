//! The error type of `tfar_debug_window`.

/// Every failure `tfar_debug_window` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation asked for is not offered by this crate.
    #[error("tfar_debug_window does not offer the operation `{operation}`")]
    UnsupportedOperation {
        /// The name of the operation that was asked for.
        operation: &'static str,
    },
}

/// The result of a fallible `tfar_debug_window` operation.
pub type Result<T> = core::result::Result<T, Error>;
