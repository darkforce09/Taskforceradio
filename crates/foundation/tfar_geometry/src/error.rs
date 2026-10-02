//! The error type of `tfar_geometry`.

/// Every failure `tfar_geometry` reports.
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

/// The result of a fallible `tfar_geometry` operation.
pub type Result<T> = core::result::Result<T, Error>;
