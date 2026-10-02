//! The error type of `tfar_plugin_settings_1_0`.

/// Every failure `tfar_plugin_settings_1_0` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation asked for is not offered by this crate.
    #[error("tfar_plugin_settings_1_0 does not offer the operation `{operation}`")]
    UnsupportedOperation {
        /// The name of the operation that was asked for.
        operation: &'static str,
    },
}

/// The result of a fallible `tfar_plugin_settings_1_0` operation.
pub type Result<T> = core::result::Result<T, Error>;
