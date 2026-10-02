//! `MuteTracker`: the clients this plugin muted in TeamSpeak.

use std::collections::BTreeSet;

use tfar_client_state_1_0::prelude::TeamspeakClientId;

use crate::mute::killed_mute_matrix::MuteChanges;

#[cfg(test)]
#[path = "tests/mute_tracker.rs"]
mod tests;

/// The clients this plugin has muted, so a mute or unmute request goes to TeamSpeak only when
/// it changes something and "unmute all" unmutes exactly these (`TeamspeakServerData::
/// mutedClients`, `reference/tfar_1_0/ts/src/Teamspeak.cpp:17-45`, `:131-205`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MuteTracker {
    muted: BTreeSet<TeamspeakClientId>,
}

impl MuteTracker {
    /// No client muted.
    pub const fn new() -> Self {
        Self {
            muted: BTreeSet::new(),
        }
    }

    /// Records `client` as muted or not; whether that changed its state, so the app sends the
    /// TeamSpeak request.
    pub fn set_muted(&mut self, client: TeamspeakClientId, mute: bool) -> bool {
        if mute {
            self.muted.insert(client)
        } else {
            self.muted.remove(&client)
        }
    }

    /// Records the mutes and unmutes of `changes` and returns those that change a client's
    /// state, which the app sends to TeamSpeak.
    pub fn apply(&mut self, changes: &MuteChanges) -> MuteChanges {
        MuteChanges {
            mute: changes
                .mute
                .iter()
                .copied()
                .filter(|client| self.set_muted(*client, true))
                .collect(),
            unmute: changes
                .unmute
                .iter()
                .copied()
                .filter(|client| self.set_muted(*client, false))
                .collect(),
        }
    }

    /// Forgets every mute and returns the clients to unmute (`unmuteAll`).
    pub fn unmute_all(&mut self) -> Vec<TeamspeakClientId> {
        core::mem::take(&mut self.muted).into_iter().collect()
    }

    /// `client` left the server: it is forgotten; whether it was muted.
    pub fn client_left(&mut self, client: TeamspeakClientId) -> bool {
        self.muted.remove(&client)
    }

    /// Whether `client` is muted by this plugin.
    pub fn is_muted(&self, client: TeamspeakClientId) -> bool {
        self.muted.contains(&client)
    }

    /// The clients muted by this plugin, in id order.
    pub fn muted(&self) -> impl Iterator<Item = TeamspeakClientId> + '_ {
        self.muted.iter().copied()
    }
}
