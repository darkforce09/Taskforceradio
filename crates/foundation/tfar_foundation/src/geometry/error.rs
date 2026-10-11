//! The error type of `tfar_foundation::geometry`.

/// Every failure `tfar_foundation::geometry` reports.
#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    /// A constructor was given a NaN or infinite coordinate; positions and directions hold only
    /// finite numbers.
    #[error("coordinates must be finite, got {coordinates:?}")]
    NonFinite {
        /// The coordinates that were given, in Arma's x (east), y (north), z (up) order.
        coordinates: [f64; 3],
    },
}

/// The result of a fallible `tfar_foundation::geometry` operation.
pub type Result<T> = core::result::Result<T, Error>;
