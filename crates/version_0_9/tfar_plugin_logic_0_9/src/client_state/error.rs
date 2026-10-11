//! The error type of `tfar_plugin_logic_0_9::client_state`.

/// Every failure `tfar_plugin_logic_0_9::client_state` reports. Each comes from creating a client's sound
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
        source: tfar_audio::dsp_filters::Error,
    },
    /// A radio effect of a client could not be built.
    #[error("a radio effect of a client cannot be built: {source}")]
    RadioEffect {
        /// The radio effect crate's error.
        #[from]
        source: crate::radio_effects::Error,
    },
    /// The client's compressor could not be built from its settings.
    #[error("the compressor of a client cannot be built: {source}")]
    Compressor {
        /// The dynamics crate's error.
        #[from]
        source: tfar_audio::dynamics::Error,
    },
}

/// The result of a fallible `tfar_plugin_logic_0_9::client_state` operation.
pub type Result<T> = core::result::Result<T, Error>;
