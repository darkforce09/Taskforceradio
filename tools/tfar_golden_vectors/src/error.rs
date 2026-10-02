//! The error type of `tfar_golden_vectors`.

/// Every failure `tfar_golden_vectors` reports: a parameter the oracle would not accept, a buffer
/// of the wrong shape, or a call the C++ shim refused.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    /// A parameter is outside the range the oracle accepts.
    #[error("the oracle parameter `{parameter}` = {value} is not {requirement}")]
    InvalidParameter {
        /// The name of the parameter.
        parameter: &'static str,
        /// The value that was given.
        value: f64,
        /// What the value must be.
        requirement: &'static str,
    },
    /// Two sequences that are compared sample by sample have different lengths.
    #[error("the compared sequences hold {reference} and {candidate} values")]
    LengthMismatch {
        /// The length of the reference sequence.
        reference: usize,
        /// The length of the candidate sequence.
        candidate: usize,
    },
    /// An interleaved buffer does not hold a whole number of frames.
    #[error("{samples} interleaved samples are not a whole number of {channels}-channel frames")]
    UnevenFrames {
        /// The number of samples in the buffer.
        samples: usize,
        /// The number of channels per frame.
        channels: usize,
    },
    /// A distance curve holds more points than the shim carries.
    #[error("the distance curve holds {given} points, more than the {maximum} the oracle carries")]
    TooManyCurvePoints {
        /// The number of points given.
        given: usize,
        /// The most points a curve may hold.
        maximum: usize,
    },
    /// The C++ shim returned a failure status.
    #[error("the oracle refused `{operation}` with status {status}")]
    OracleRefused {
        /// The shim function that refused.
        operation: &'static str,
        /// The status code it returned: 1 for an invalid argument, 2 for a C++ exception.
        status: i32,
    },
}

/// The result of a fallible `tfar_golden_vectors` operation.
pub type Result<T> = core::result::Result<T, Error>;
