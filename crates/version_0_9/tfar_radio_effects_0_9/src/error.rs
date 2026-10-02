//! The error type of `tfar_radio_effects_0_9`.

/// Every failure `tfar_radio_effects_0_9` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A filter of an effect could not be designed. The designs are fixed constants, so this
    /// means the filter library refused one of them.
    #[error("the {effect} radio effect cannot design its {filter} filter: {source}")]
    FilterDesign {
        /// The effect being built.
        effect: &'static str,
        /// Which of its filters failed.
        filter: &'static str,
        /// The filter library's error.
        source: tfar_dsp_filters::Error,
    },
}

/// The result of a fallible `tfar_radio_effects_0_9` operation.
pub type Result<T> = core::result::Result<T, Error>;
