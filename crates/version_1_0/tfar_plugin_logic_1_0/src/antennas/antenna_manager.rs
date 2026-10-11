//! `AntennaManager`: every radio tower the game reports, and the search for the best one.
//!
//! **Role:** keep the towers of `RadioTwrAdd` and `RadioTwrDel` and find the tower that best
//! relays a transmission, as `AntennaManager` (`reference/tfar_1_0/ts/src/antennaManager.cpp`).
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::antennas`; fed by the plugin app from the parsed tower
//! messages; asked by `tfar_plugin_logic_1_0::radio_routing` for every sender.
//!
//! **Signals and state:** the list of towers, behind the lock of whoever owns the manager (the
//! game link worker writes it, routing reads it).
//!
//! **Invariants:** no two towers share an id and the null id is never stored; a connection is
//! found only through a tower both ends reach, with a loss below 1.

use tfar_foundation::geometry::Position;

use crate::antennas::antenna::Antenna;
use crate::antennas::antenna_connection::AntennaConnection;
use crate::antennas::net_id::NetId;

#[cfg(test)]
#[path = "tests/antenna_manager.rs"]
mod tests;

/// The radio towers of the current game, as `AntennaManager` in
/// `reference/tfar_1_0/ts/src/antennaManager.cpp`.
///
/// The manager holds no lock: its owner keeps it behind the lock of the state it belongs to, and
/// [`AntennaManager::find_connection`] only reads.
#[derive(Clone, Debug, Default)]
pub struct AntennaManager {
    antennas: Vec<Antenna>,
}

impl AntennaManager {
    /// A manager with no towers.
    pub const fn new() -> Self {
        Self {
            antennas: Vec::new(),
        }
    }

    /// Adds `antennas` (`RadioTwrAdd`): a tower whose id is already known replaces the known one
    /// in place, a new id is appended, and a tower with the null id is ignored.
    pub fn add(&mut self, antennas: impl IntoIterator<Item = Antenna>) {
        for antenna in antennas {
            if antenna.id().is_null() {
                continue;
            }
            match self
                .antennas
                .iter_mut()
                .find(|known| known.id() == antenna.id())
            {
                Some(known) => *known = antenna,
                None => self.antennas.push(antenna),
            }
        }
    }

    /// Removes the towers named by `ids` (`RadioTwrDel`); the null id and unknown ids are
    /// ignored.
    pub fn remove(&mut self, ids: impl IntoIterator<Item = NetId>) {
        for id in ids {
            if id.is_null() {
                continue;
            }
            if let Some(index) = self.antennas.iter().position(|known| known.id() == id) {
                self.antennas.remove(index);
            }
        }
    }

    /// Forgets every tower.
    pub fn clear(&mut self) {
        self.antennas.clear();
    }

    /// The towers, in the order they were first added.
    pub fn antennas(&self) -> &[Antenna] {
        &self.antennas
    }

    /// The number of towers.
    pub fn len(&self) -> usize {
        self.antennas.len()
    }

    /// Whether there is no tower.
    pub fn is_empty(&self) -> bool {
        self.antennas.is_empty()
    }

    /// The best tower between a sender at `sender` with a radio of `sender_range` metres and a
    /// receiver at `receiver` (`findConnection`, `antennaManager.cpp:24-46`): among the towers
    /// the sender reaches and that reach the receiver, the one with the lowest
    /// [`Antenna::connection_loss`], provided it is below 1. The first tower wins a tie.
    /// `None` when no tower qualifies.
    pub fn find_connection(
        &self,
        sender: Position,
        sender_range: f64,
        receiver: Position,
    ) -> Option<AntennaConnection> {
        let mut best: Option<AntennaConnection> = None;
        for antenna in &self.antennas {
            if !antenna.is_reached_by(sender, sender_range) || !antenna.reaches(receiver) {
                continue;
            }
            let loss = antenna.connection_loss(sender, sender_range, receiver);
            let lowest = best.map_or(1.0, |connection| connection.loss());
            if loss < lowest {
                best = Some(AntennaConnection::new(*antenna, loss));
            }
        }
        best
    }
}
