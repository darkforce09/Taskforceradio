//! `MuteTracker`: the clients this plugin has muted, and the requests a wanted state needs.
//!
//! **Role:** turn each wanted mute state into the TeamSpeak requests that reach it from what the
//! plugin itself asked for, so a request goes out only on a change and an unmute only to a
//! client this plugin muted.
//!
//! **Position:** the plugin app keeps one per server connection beside the client state; the
//! mute decisions in this folder feed it, and the plugin sends the `MuteDecision`s it returns
//! with `requestMuteClients` and `requestUnmuteClients` after releasing its lock.
//!
//! **Signals and state:** the set of muted client ids, changed on the threads that make mute
//! decisions, under the plugin's server lock.
//!
//! **Invariants:** a client id appears in at most one list of a decision; client id 0 is never
//! in one; `unmute_all` unmutes exactly the tracked set and empties it.

use std::collections::BTreeSet;

use tfar_client_state_0_9::ClientId;

#[cfg(test)]
#[path = "tests/mute_tracker.rs"]
mod tests;

/// The TeamSpeak requests one or more mute decisions need.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MuteDecision {
    /// The clients to mute (`requestMuteClients`).
    pub mute: Vec<ClientId>,
    /// The clients to unmute (`requestUnmuteClients`).
    pub unmute: Vec<ClientId>,
}

impl MuteDecision {
    /// Whether no request is needed.
    pub fn is_empty(&self) -> bool {
        self.mute.is_empty() && self.unmute.is_empty()
    }

    /// Appends the requests of `other`.
    pub fn merge(&mut self, other: Self) {
        self.mute.extend(other.mute);
        self.unmute.extend(other.unmute);
    }
}

/// The clients this plugin muted on one server connection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MuteTracker {
    muted: BTreeSet<ClientId>,
}

impl MuteTracker {
    /// A tracker that has muted nobody.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether this plugin muted `client_id`.
    pub fn is_muted(&self, client_id: ClientId) -> bool {
        self.muted.contains(&client_id)
    }

    /// The clients this plugin muted, in id order.
    pub fn muted(&self) -> impl Iterator<Item = ClientId> + '_ {
        self.muted.iter().copied()
    }

    /// The request that brings `client_id` to the wanted state `should_mute`: a mute when it is
    /// not muted yet, an unmute when this plugin muted it, nothing otherwise or for client id 0.
    pub fn decide(&mut self, client_id: ClientId, should_mute: bool) -> MuteDecision {
        let mut decision = MuteDecision::default();
        if !client_id.names_a_client() {
            return decision;
        }
        if should_mute {
            if self.muted.insert(client_id) {
                decision.mute.push(client_id);
            }
        } else if self.muted.remove(&client_id) {
            decision.unmute.push(client_id);
        }
        decision
    }

    /// Unmutes every client this plugin muted (`unmuteAll`,
    /// `reference/tfar_0_9_12/ts/src/plugin.cpp:1220`, which unmutes every client on the
    /// server instead).
    pub fn unmute_all(&mut self) -> MuteDecision {
        MuteDecision {
            mute: Vec::new(),
            unmute: std::mem::take(&mut self.muted).into_iter().collect(),
        }
    }

    /// Forgets `client_id` without a request: TeamSpeak refused the mute, or the client left
    /// and its id may come back on someone else.
    pub fn forget(&mut self, client_id: ClientId) {
        self.muted.remove(&client_id);
    }

    /// Forgets every muted client not in `present`, the server's client list.
    pub fn retain_present(&mut self, present: &[ClientId]) {
        self.muted.retain(|client_id| present.contains(client_id));
    }
}
