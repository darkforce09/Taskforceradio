//! The error type of `tfar_audio::dsp_filters`.

/// Every failure `tfar_audio::dsp_filters` reports. Each one comes from filter construction; processing
/// never fails.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    /// The sample rate is not a finite number above zero.
    #[error("the sample rate {sample_rate_hz} Hz is not a finite number above 0")]
    InvalidSampleRate {
        /// The sample rate that was given, in hertz.
        sample_rate_hz: f64,
    },
    /// A design frequency (a cutoff or a band edge) is not strictly between 0 and Nyquist.
    #[error(
        "the {frequency_role} {frequency_hz} Hz is not strictly between 0 and the Nyquist \
         frequency {nyquist_hz} Hz"
    )]
    FrequencyOutOfRange {
        /// Which frequency is out of range: `"cutoff"`, `"lower band edge"` or `"upper band edge"`.
        frequency_role: &'static str,
        /// The frequency that was given or derived, in hertz.
        frequency_hz: f64,
        /// Half the sample rate, in hertz.
        nyquist_hz: f64,
    },
    /// A band-pass width is not a finite number above zero, or so small that the two band edges
    /// coincide.
    #[error("the band-pass width {width_hz} Hz around {centre_hz} Hz gives no band")]
    InvalidBandWidth {
        /// The centre frequency that was given, in hertz.
        centre_hz: f64,
        /// The width that was given, in hertz.
        width_hz: f64,
    },
    /// An RBJ quality factor is not a finite number above zero.
    #[error("the quality factor {quality} is not a finite number above 0")]
    InvalidQuality {
        /// The quality factor that was given.
        quality: f64,
    },
    /// A Butterworth order is outside `1..=MAX_BUTTERWORTH_ORDER`.
    #[error("the Butterworth order {order} is outside 1..={maximum}")]
    InvalidOrder {
        /// The order that was given.
        order: usize,
        /// The highest order the crate designs.
        maximum: usize,
    },
    /// The design library produced no usable second-order sections (none at all, a zero leading
    /// denominator coefficient, or a coefficient that is not finite).
    #[error("the filter design produced no usable second-order sections: {reason}")]
    DesignFailed {
        /// What was wrong with the design output.
        reason: &'static str,
    },
}

/// The result of a fallible `tfar_audio::dsp_filters` operation.
pub type Result<T> = core::result::Result<T, Error>;
