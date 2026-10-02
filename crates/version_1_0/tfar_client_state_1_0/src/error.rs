//! The error type of `tfar_client_state_1_0`.

/// Every failure `tfar_client_state_1_0` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An HRTF source was asked for while no HRIR sphere is loaded.
    #[error("the HRTF source `{key}` needs the HRIR sphere, which is not loaded")]
    NoHrirLibrary {
        /// The key of the source.
        key: String,
    },
    /// A per-sender filter could not be designed.
    #[error("a sender filter cannot be designed: {0}")]
    Filter(#[from] tfar_dsp_filters::Error),
    /// A radio effect could not be built.
    #[error("a radio effect cannot be built: {0}")]
    RadioEffect(#[from] tfar_radio_effects_1_0::Error),
    /// A compressor could not be built.
    #[error("a compressor cannot be built: {0}")]
    Compressor(#[from] tfar_dynamics::Error),
}

/// The result of a fallible `tfar_client_state_1_0` operation.
pub type Result<T> = core::result::Result<T, Error>;
