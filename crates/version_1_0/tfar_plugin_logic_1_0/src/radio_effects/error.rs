//! The error type of `tfar_plugin_logic_1_0::radio_effects`.

/// Every failure `tfar_plugin_logic_1_0::radio_effects` reports.
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
        source: tfar_audio::dsp_filters::Error,
    },
}

/// The result of a fallible `tfar_plugin_logic_1_0::radio_effects` operation.
pub type Result<T> = core::result::Result<T, Error>;
