//! `ClientDirectory`: the clients of one TeamSpeak server connection, keyed by nickname.
//!
//! **Role:** keep one [`ClientData`] per TeamSpeak client, found by its nickname (the name the
//! game uses) or by its client id, and mark which one is the local player, as `serverData`
//! (`reference/tfar_1_0/ts/src/serverData.cpp`).
//!
//! **Position:** a module of `tfar_client_state_1_0`; part of [`crate::SessionState`]; fed by the
//! plugin app's TeamSpeak client events; read by every apply function and the routing crate.
//!
//! **Signals and state:** a map by nickname and an index by client id, behind the caller's
//! session lock; the TeamSpeak event thread writes it.
//!
//! **Invariants:** clients are keyed by the real nickname string, never a hash, so two nicknames
//! never share an entry; the id index and the map always agree (each id names exactly one entry
//! whose nickname is its key); a rename moves the entry with its state; the local player's entry
//! is never removed by a leave.

use std::collections::{BTreeMap, HashMap};

use tfar_plugin_commands_1_0::TeamspeakClientId;

use crate::client_data::ClientData;

#[cfg(test)]
#[path = "tests/client_directory.rs"]
mod tests;

/// The clients of one server connection.
#[derive(Debug, Default)]
pub struct ClientDirectory {
    by_nickname: BTreeMap<String, ClientData>,
    nickname_by_id: HashMap<TeamspeakClientId, String>,
    own_client_id: Option<TeamspeakClientId>,
}

impl ClientDirectory {
    /// A directory with no client.
    pub fn new() -> Self {
        Self::default()
    }

    /// A client joined or became visible (`clientJoined`): a known id keeps its state under the
    /// new nickname; a new id gets a fresh entry. A different client holding the nickname is
    /// dropped, as TeamSpeak gives every client its own name.
    pub fn client_joined(&mut self, client_id: TeamspeakClientId, nickname: &str) {
        if self.nickname_by_id.contains_key(&client_id) {
            self.client_renamed(client_id, nickname);
            return;
        }
        self.drop_holder_of(nickname);
        self.nickname_by_id.insert(client_id, nickname.to_owned());
        self.by_nickname
            .insert(nickname.to_owned(), ClientData::new(client_id, nickname));
    }

    /// The client `client_id` is now called `nickname` (`clientUpdated`); its state moves with
    /// it. An unknown id is ignored.
    pub fn client_renamed(&mut self, client_id: TeamspeakClientId, nickname: &str) {
        let Some(old_nickname) = self.nickname_by_id.get(&client_id).cloned() else {
            return;
        };
        if old_nickname == nickname {
            return;
        }
        let Some(mut client) = self.by_nickname.remove(&old_nickname) else {
            return;
        };
        self.drop_holder_of(nickname);
        client.set_nickname(nickname.to_owned());
        self.nickname_by_id.insert(client_id, nickname.to_owned());
        self.by_nickname.insert(nickname.to_owned(), client);
    }

    /// The client `client_id` left (`clientLeft`); the local player stays.
    pub fn client_left(&mut self, client_id: TeamspeakClientId) {
        if self.own_client_id == Some(client_id) {
            return;
        }
        if let Some(nickname) = self.nickname_by_id.remove(&client_id) {
            self.by_nickname.remove(&nickname);
        }
    }

    /// Every client left at once (a channel switch, the original's client id -2); the local
    /// player stays.
    pub fn every_client_left(&mut self) {
        let own = self
            .own_client_id
            .and_then(|id| self.nickname_by_id.get(&id).cloned())
            .and_then(|nickname| self.by_nickname.remove(&nickname));
        self.by_nickname.clear();
        self.nickname_by_id.clear();
        if let Some(own) = own {
            self.nickname_by_id
                .insert(own.client_id(), own.nickname().to_owned());
            self.by_nickname.insert(own.nickname().to_owned(), own);
        }
    }

    /// Marks `client_id` as the local player, adding the entry when it is new (the connect
    /// handling, `Teamspeak.cpp:317`).
    pub fn set_own_client(&mut self, client_id: TeamspeakClientId, nickname: &str) {
        self.client_joined(client_id, nickname);
        self.own_client_id = Some(client_id);
    }

    /// The local player's client id.
    pub const fn own_client_id(&self) -> Option<TeamspeakClientId> {
        self.own_client_id
    }

    /// The local player's entry.
    pub fn own(&self) -> Option<&ClientData> {
        self.by_id(self.own_client_id?)
    }

    /// The local player's entry, to change.
    pub fn own_mut(&mut self) -> Option<&mut ClientData> {
        self.by_id_mut(self.own_client_id?)
    }

    /// Whether `client` is the local player.
    pub fn is_own(&self, client: &ClientData) -> bool {
        self.own_client_id == Some(client.client_id())
    }

    /// The client called `nickname`.
    pub fn get(&self, nickname: &str) -> Option<&ClientData> {
        self.by_nickname.get(nickname)
    }

    /// The client called `nickname`, to change.
    pub fn get_mut(&mut self, nickname: &str) -> Option<&mut ClientData> {
        self.by_nickname.get_mut(nickname)
    }

    /// The client `client_id`.
    pub fn by_id(&self, client_id: TeamspeakClientId) -> Option<&ClientData> {
        self.by_nickname.get(self.nickname_by_id.get(&client_id)?)
    }

    /// The client `client_id`, to change.
    pub fn by_id_mut(&mut self, client_id: TeamspeakClientId) -> Option<&mut ClientData> {
        self.by_nickname
            .get_mut(self.nickname_by_id.get(&client_id)?)
    }

    /// Every client, in nickname order.
    pub fn iter(&self) -> impl Iterator<Item = &ClientData> {
        self.by_nickname.values()
    }

    /// Every client, in nickname order, to change.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ClientData> {
        self.by_nickname.values_mut()
    }

    /// The number of clients.
    pub fn len(&self) -> usize {
        self.by_nickname.len()
    }

    /// Whether there is no client.
    pub fn is_empty(&self) -> bool {
        self.by_nickname.is_empty()
    }

    /// Removes the entry holding `nickname`, if any, with its id.
    fn drop_holder_of(&mut self, nickname: &str) {
        if let Some(holder) = self.by_nickname.remove(nickname) {
            self.nickname_by_id.remove(&holder.client_id());
            if self.own_client_id == Some(holder.client_id()) {
                self.own_client_id = None;
            }
        }
    }
}
