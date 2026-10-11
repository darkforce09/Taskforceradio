//! The error type of `tfar_audio::spatial_audio`.

/// Every failure `tfar_audio::spatial_audio` reports.
///
/// Each variant carries the values a caller needs to see which input is wrong. Only the
/// constructors fail; the gain functions are total.
#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    /// A distance curve was given no point at all.
    #[error("a distance curve needs at least one point, but none were given")]
    EmptyDistanceCurve,
    /// A distance curve was given more points than it can hold.
    #[error("a distance curve holds at most {maximum} points, but {given} were given")]
    TooManyCurvePoints {
        /// The points that were given.
        given: usize,
        /// The most points a curve holds.
        maximum: usize,
    },
    /// A curve point has a distance that is NaN, infinite or negative.
    #[error(
        "distance curve point {index} has the distance {distance}, which is not finite and >= 0"
    )]
    InvalidCurveDistance {
        /// The position of the point in the list that was given.
        index: usize,
        /// The distance that was given.
        distance: f64,
    },
    /// A curve point has a gain that is NaN, infinite or negative.
    #[error("distance curve point {index} has the gain {gain}, which is not finite and >= 0")]
    InvalidCurveGain {
        /// The position of the point in the list that was given.
        index: usize,
        /// The gain that was given.
        gain: f64,
    },
    /// A curve point's distance is not greater than the distance of the point before it.
    #[error(
        "distance curve point {index} has the distance {distance}, which is not greater than the \
         previous distance {previous_distance}"
    )]
    UnsortedCurvePoints {
        /// The position of the out-of-order point in the list that was given.
        index: usize,
        /// The distance of the point before it.
        previous_distance: f64,
        /// The distance of the out-of-order point.
        distance: f64,
    },
    /// An emitter cone's angles are not `0 <= inner <= outer <= 2π`.
    #[error(
        "an emitter cone needs 0 <= inner angle <= outer angle <= 2π radians, got inner \
         {inner_angle} and outer {outer_angle}"
    )]
    InvalidConeAngles {
        /// The inner angle that was given, in radians.
        inner_angle: f64,
        /// The outer angle that was given, in radians.
        outer_angle: f64,
    },
    /// An emitter cone's volume is not finite and within 0 to 2.
    #[error("an emitter cone volume must lie within 0 to 2, got the {which} volume {volume}")]
    InvalidConeVolume {
        /// Which volume is wrong: `"inner"` or `"outer"`.
        which: &'static str,
        /// The volume that was given.
        volume: f64,
    },
}

/// The result of a fallible `tfar_audio::spatial_audio` operation.
pub type Result<T> = core::result::Result<T, Error>;
