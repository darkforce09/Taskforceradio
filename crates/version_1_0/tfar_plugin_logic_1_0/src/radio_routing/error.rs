//! The error type of `tfar_plugin_logic_1_0::radio_routing`.

/// Every failure `tfar_plugin_logic_1_0::radio_routing` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation asked for is not offered by this crate.
    #[error("tfar_plugin_logic_1_0::radio_routing does not offer the operation `{operation}`")]
    UnsupportedOperation {
        /// The name of the operation that was asked for.
        operation: &'static str,
    },
}

/// The result of a fallible `tfar_plugin_logic_1_0::radio_routing` operation.
pub type Result<T> = core::result::Result<T, Error>;
