//! The serious-mode auto-mute in TeamSpeak: decide under the server lock with
//! `tfar_plugin_logic_1_0::radio_routing`, record in the `MuteTracker`, send after the lock is released.
//!
//! **Role:** `setGameClientMuteStatus` (`reference/tfar_1_0/ts/src/plugin.cpp:68-94`) for one
//! client, for every client, and the killed-player mute matrix.
//!
//! **Position:** methods of `PluginRuntime`; called after a `POS`, a `VOLUME` or tangent plugin
//! command, a `KILLED`, a serious-mode change and the 4 s service sweep.
//!
//! **Signals and state:** the server state's session and mute tracker, under its lock.
//!
//! **Invariants:** a mute or unmute request reaches TeamSpeak only when it changes what this
//! plugin muted; the local client is never muted; no request while dormant.

use std::time::Instant;

use tfar_plugin_logic_1_0::client_state::prelude::TeamspeakClientId;
use tfar_plugin_logic_1_0::plugin_settings::PluginSettings;
use tfar_plugin_logic_1_0::radio_routing::{
    MuteChanges, RoutingContext, killed_mute_matrix, mute_decision,
};
use tfar_ts3_host::{ClientId, ServerConnectionHandlerId};

use crate::runtime::plugin_runtime::PluginRuntime;
use crate::runtime::server_state::{ServerState, host_client_id, state_client_id};

/// Decides `client`'s mute under the lock and records it; the change to send, if any.
/// `heard_over_radio` is the routing answer when the caller already has it.
pub fn decide_client_mute(
    state: &mut ServerState,
    settings: &PluginSettings,
    client: TeamspeakClientId,
    heard_over_radio: Option<bool>,
    now: Instant,
) -> Option<(TeamspeakClientId, bool)> {
    let decision = {
        let context = RoutingContext {
            session: &state.session,
            settings,
            antennas: &state.antennas,
            now,
        };
        let target = state.session.clients.by_id(client);
        mute_decision(&context, target, state.is_serious_mode, heard_over_radio)?
    };
    if decision.reset_voices
        && let Some(target) = state.session.clients.by_id_mut(client)
    {
        target.effects.reset_voices();
    }
    state
        .mutes
        .set_muted(client, decision.mute)
        .then_some((client, decision.mute))
}

/// The mutes of a `KILLED` message over `channel_clients`, recorded; the changes to send.
pub fn decide_killed_mutes(
    state: &mut ServerState,
    settings: &PluginSettings,
    channel_clients: &[ClientId],
    now: Instant,
) -> MuteChanges {
    let clients: Vec<TeamspeakClientId> = channel_clients
        .iter()
        .copied()
        .map(state_client_id)
        .collect();
    let changes = {
        let context = RoutingContext {
            session: &state.session,
            settings,
            antennas: &state.antennas,
            now,
        };
        killed_mute_matrix(&context, state.is_serious_mode, &clients)
    };
    state.mutes.apply(&changes)
}

impl PluginRuntime {
    /// Sends recorded mute changes to TeamSpeak.
    pub fn send_mute_changes(
        &self,
        server: ServerConnectionHandlerId,
        changes: impl IntoIterator<Item = (TeamspeakClientId, bool)>,
    ) {
        let mut mute = Vec::new();
        let mut unmute = Vec::new();
        for (client, is_muted) in changes {
            if is_muted {
                mute.push(host_client_id(client));
            } else {
                unmute.push(host_client_id(client));
            }
        }
        if mute.is_empty() && unmute.is_empty() {
            return;
        }
        let Some(ts) = self.act() else {
            return;
        };
        ts.mute_clients(server, &mute);
        ts.unmute_clients(server, &unmute);
    }

    /// Sends a killed-player mute matrix.
    pub fn send_mute_matrix(&self, server: ServerConnectionHandlerId, changes: &MuteChanges) {
        let muted = changes.mute.iter().map(|client| (*client, true));
        let unmuted = changes.unmute.iter().map(|client| (*client, false));
        self.send_mute_changes(server, muted.chain(unmuted));
    }

    /// Recomputes and sends the mute of every known client of `server`.
    pub fn refresh_every_mute(&self, server: ServerConnectionHandlerId) {
        let Some(state) = self.servers.get(server) else {
            return;
        };
        let settings = self.settings();
        let now = Instant::now();
        let changes: Vec<_> = {
            let mut state = state.lock();
            let clients: Vec<TeamspeakClientId> = state
                .session
                .clients
                .iter()
                .map(tfar_plugin_logic_1_0::client_state::prelude::ClientData::client_id)
                .collect();
            clients
                .into_iter()
                .filter_map(|client| decide_client_mute(&mut state, &settings, client, None, now))
                .collect()
        };
        self.send_mute_changes(server, changes);
    }

    /// A client left `server`: it is forgotten and, when this plugin muted it, unmuted.
    pub fn forget_client(&self, server: ServerConnectionHandlerId, client: ClientId) {
        let Some(state) = self.servers.get(server) else {
            return;
        };
        let was_muted = {
            let mut state = state.lock();
            let id = state_client_id(client);
            state.session.clients.client_left(id);
            state.client_metadata.remove(&id);
            state.mutes.client_left(id)
        };
        if was_muted && let Some(ts) = self.act() {
            ts.unmute_clients(server, &[client]);
        }
    }
}
