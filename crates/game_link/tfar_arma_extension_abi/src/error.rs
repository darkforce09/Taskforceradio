//! The error type of `tfar_arma_extension_abi`.

/// Every failure `tfar_arma_extension_abi` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation asked for is not offered by this crate.
    #[error("tfar_arma_extension_abi does not offer the operation `{operation}`")]
    UnsupportedOperation {
        /// The name of the operation that was asked for.
        operation: &'static str,
    },
}

/// The result of a fallible `tfar_arma_extension_abi` operation.
pub type Result<T> = core::result::Result<T, Error>;
