//! The error type of `tfar_audio::dynamics`.

/// Every failure `tfar_audio::dynamics` reports. Each one comes from building a compressor; processing
/// never fails.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    /// A compressor setting is outside its range: the sample rate, the ratio and the two time
    /// constants must be finite and above zero, the threshold finite.
    #[error("the compressor setting `{setting}` = {value} is outside its range ({requirement})")]
    InvalidSetting {
        /// The name of the setting, as the `CompressorSettings` field is named.
        setting: &'static str,
        /// The value that was given.
        value: f64,
        /// What the setting must be.
        requirement: &'static str,
    },
}

/// The result of a fallible `tfar_audio::dynamics` operation.
pub type Result<T> = core::result::Result<T, Error>;
