//! The error type of `tfar_antennas_1_0`.

use crate::net_id::NetId;

/// Every failure `tfar_antennas_1_0` reports.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A tower's range is negative or not finite.
    #[error("the radio tower {id} has the invalid range {range}")]
    InvalidRange {
        /// The tower's id.
        id: NetId,
        /// The range it was given.
        range: f64,
    },
}

/// The result of a fallible `tfar_antennas_1_0` operation.
pub type Result<T> = core::result::Result<T, Error>;
