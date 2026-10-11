//! The error type of `tfar_plugin_logic_1_0::antennas`.

use crate::antennas::net_id::NetId;

/// Every failure `tfar_plugin_logic_1_0::antennas` reports.
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

/// The result of a fallible `tfar_plugin_logic_1_0::antennas` operation.
pub type Result<T> = core::result::Result<T, Error>;
