//! The error type of `tfar_ts3_settings_db`.

/// Every failure `tfar_ts3_settings_db` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation asked for is not offered by this crate.
    #[error("tfar_ts3_settings_db does not offer the operation `{operation}`")]
    UnsupportedOperation {
        /// The name of the operation that was asked for.
        operation: &'static str,
    },
}

/// The result of a fallible `tfar_ts3_settings_db` operation.
pub type Result<T> = core::result::Result<T, Error>;
