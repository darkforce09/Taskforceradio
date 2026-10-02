//! The error type of `tfar_playback_mixer`.

/// Every failure `tfar_playback_mixer` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation asked for is not offered by this crate.
    #[error("tfar_playback_mixer does not offer the operation `{operation}`")]
    UnsupportedOperation {
        /// The name of the operation that was asked for.
        operation: &'static str,
    },
}

/// The result of a fallible `tfar_playback_mixer` operation.
pub type Result<T> = core::result::Result<T, Error>;
