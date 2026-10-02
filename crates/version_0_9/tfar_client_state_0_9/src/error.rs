//! The error type of `tfar_client_state_0_9`.

/// Every failure `tfar_client_state_0_9` reports. Each comes from creating a client's sound
/// state; applying a command never fails.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A filter of a client's sound state could not be designed. The designs are fixed, or for
    /// the vehicle low-pass bounded by the isolation clamp, so the filter library refused one.
    #[error("the {filter} filter of a client cannot be designed: {source}")]
    FilterDesign {
        /// Which filter failed.
        filter: &'static str,
        /// The filter library's error.
        source: tfar_dsp_filters::Error,
    },
    /// A radio effect of a client could not be built.
    #[error("a radio effect of a client cannot be built: {source}")]
    RadioEffect {
        /// The radio effect crate's error.
        #[from]
        source: tfar_radio_effects_0_9::Error,
    },
    /// The client's compressor could not be built from its settings.
    #[error("the compressor of a client cannot be built: {source}")]
    Compressor {
        /// The dynamics crate's error.
        #[from]
        source: tfar_dynamics::Error,
    },
}

/// The result of a fallible `tfar_client_state_0_9` operation.
pub type Result<T> = core::result::Result<T, Error>;
