//! `AntennaConnection`: the best tower between a sender and a receiver, and its loss.

use crate::antennas::antenna::Antenna;

#[cfg(test)]
#[path = "tests/antenna_connection.rs"]
mod tests;

/// The loss the original gives "no connection" (`AntennaConnection::isNull`,
/// `reference/tfar_1_0/ts/src/antennaManager.h:99`): a value no real connection reaches, since a
/// connection's loss is always below 1.
///
/// Callers that take the minimum of an antenna loss and a distance error, as the voice pipeline
/// does for the radio effect's error level, map a missing connection to this value with
/// [`loss_or_no_connection`], which keeps the original's cap at 7 exactly.
pub const NO_CONNECTION_LOSS: f64 = 7.0;

/// A transmission relayed through one tower, as `AntennaConnection` in
/// `reference/tfar_1_0/ts/src/antennaManager.h:94-126`; "no connection" is `None` where it is
/// used.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AntennaConnection {
    antenna: Antenna,
    loss: f64,
}

impl AntennaConnection {
    /// The connection through `antenna` with `loss`.
    pub(crate) const fn new(antenna: Antenna, loss: f64) -> Self {
        Self { antenna, loss }
    }

    /// The tower the transmission hops through.
    pub const fn antenna(&self) -> &Antenna {
        &self.antenna
    }

    /// The connection's signal loss, at least 0 and below 1.
    pub const fn loss(&self) -> f64 {
        self.loss
    }
}

/// The loss of `loss` (an antenna connection's loss, or `None` without one), with "no
/// connection" read as [`NO_CONNECTION_LOSS`].
pub fn loss_or_no_connection(loss: Option<f64>) -> f64 {
    loss.unwrap_or(NO_CONNECTION_LOSS)
}
