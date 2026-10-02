//! `ReceivingTransmissions`: the transmissions the local player receives, one per sender.

use std::collections::BTreeMap;

use tfar_plugin_commands_1_0::TeamspeakClientId;

use crate::client_data::ClientData;

#[cfg(test)]
#[path = "tests/receiving_transmissions.rs"]
mod tests;

/// The senders the local player hears on a radio now, each with the frequency they transmit on.
///
/// The original counts with `receivingTransmission` and a set of frequencies
/// (`reference/tfar_1_0/ts/src/plugin.cpp:984-1016`), which drift apart when two senders share a
/// frequency; here each sender counts once and leaves with its own release.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ReceivingTransmissions {
    by_sender: BTreeMap<TeamspeakClientId, String>,
}

impl ReceivingTransmissions {
    /// Starts receiving `sender` on `frequency`; whether the set changed.
    pub(crate) fn start(&mut self, sender: TeamspeakClientId, frequency: &str) -> bool {
        if self.by_sender.contains_key(&sender) {
            return false;
        }
        self.by_sender.insert(sender, frequency.to_owned());
        true
    }

    /// Stops receiving `sender`; whether the set changed.
    pub(crate) fn stop(&mut self, sender: TeamspeakClientId) -> bool {
        self.by_sender.remove(&sender).is_some()
    }
}

impl ClientData {
    /// How many transmissions the local player receives now (`receivingTransmission`).
    pub fn receiving_transmission_count(&self) -> usize {
        self.receiving.by_sender.len()
    }

    /// Whether the local player receives a transmission now: the second character of the `POS`
    /// reply.
    pub fn is_receiving_transmission(&self) -> bool {
        !self.receiving.by_sender.is_empty()
    }

    /// The frequencies received now (`receivingFrequencies`), one per sender.
    pub fn receiving_frequencies(&self) -> impl Iterator<Item = &str> {
        self.receiving.by_sender.values().map(String::as_str)
    }

    /// The senders received now.
    pub fn receiving_senders(&self) -> impl Iterator<Item = TeamspeakClientId> + '_ {
        self.receiving.by_sender.keys().copied()
    }
}
