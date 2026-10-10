//! [`ServerState`]: everything the plugin keeps for one TeamSpeak server connection, and
//! [`ServerRegistry`], the map of them.
//!
//! **Role:** bundle the 1.0 session (game data and clients), the mute tracker, the radio towers,
//! the voice processor with its stereo scratch, the client metadata the voice gate reads, the
//! channel cache and the nickname and channel to restore, so one lock guards all of it.
//!
//! **Position:** owned by `runtime::PluginRuntime`; locked by the game link threads, TeamSpeak's
//! main thread, the service thread and the audio callbacks, which wait for it as the original
//! waits on its critical section.
//!
//! **Signals and state:** one `parking_lot::Mutex<ServerState>` per connection inside an `Arc`, in
//! a `parking_lot::RwLock` map written only when a connection is established or closed.
//!
//! **Invariants:** no TeamSpeak call happens, and no other lock is taken, while a server lock is
//! held, so an audio callback waiting for it waits only for a short decision; the audio callbacks
//! never wait for the registry; every scratch buffer is sized before the first frame.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::{Mutex, RwLock};
use tfar_antennas_1_0::AntennaManager;
use tfar_client_state_1_0::prelude::{SessionState, TeamspeakClientId};
use tfar_radio_routing_1_0::MuteTracker;
use tfar_sample_buffer::StereoBuffer;
use tfar_ts3_host::{AudioDiagnostics, ChannelId, ClientId, ServerConnectionHandlerId};
use tfar_voice_pipeline_1_0::VoiceProcessor;

/// The largest voice block, in frames, the scratch buffers hold without growing: 100 ms at
/// 48 kHz, ten of TeamSpeak's usual 10 ms blocks.
pub const MAX_BLOCK_FRAMES: usize = 4_800;

/// The plugin's state for one server connection; see the module header.
#[derive(Debug)]
pub struct ServerState {
    /// The game data and the clients of this connection.
    pub session: SessionState,
    /// The clients this plugin muted.
    pub mutes: MuteTracker,
    /// The radio towers the game reported.
    pub antennas: AntennaManager,
    /// The 1.0 voice chain and its scratch.
    pub voice: VoiceProcessor,
    /// The post-process callback's stereo copy of a block.
    pub playback_stereo: StereoBuffer,
    /// The captured-voice callback's stereo copy of the microphone.
    pub captured_stereo: StereoBuffer,
    /// The last metadata read of each client, which the voice gate reads when its cache expired.
    pub client_metadata: BTreeMap<TeamspeakClientId, String>,
    /// The channels of the server with their names, in TeamSpeak's order.
    pub channels: Vec<(ChannelId, String)>,
    /// The channel the local client is in.
    pub own_channel: Option<ChannelId>,
    /// The channel to return to at game end, saved when the plugin moved into the serious channel.
    pub original_channel: Option<ChannelId>,
    /// The nickname to restore at game end, saved at the first rename of a game.
    pub original_nickname: Option<String>,
    /// The local client is in the serious channel.
    pub is_serious_mode: bool,
}

impl ServerState {
    /// An empty state whose game data carries the push-to-talk delay read at init. Allocates
    /// every scratch buffer: call it off the audio thread.
    pub fn new(push_to_talk_delay: Duration) -> Self {
        let mut session = SessionState::new();
        session.game.push_to_talk_delay = push_to_talk_delay;
        Self {
            session,
            mutes: MuteTracker::new(),
            antennas: AntennaManager::new(),
            voice: VoiceProcessor::new(MAX_BLOCK_FRAMES),
            playback_stereo: StereoBuffer::with_frames(MAX_BLOCK_FRAMES),
            captured_stereo: StereoBuffer::with_frames(MAX_BLOCK_FRAMES),
            client_metadata: BTreeMap::new(),
            channels: Vec::new(),
            own_channel: None,
            original_channel: None,
            original_nickname: None,
            is_serious_mode: false,
        }
    }
}

/// The client id of the client state for a TeamSpeak client id.
pub const fn state_client_id(client: ClientId) -> TeamspeakClientId {
    TeamspeakClientId(client.0)
}

/// The TeamSpeak client id for a client id of the client state.
pub const fn host_client_id(client: TeamspeakClientId) -> ClientId {
    ClientId(client.0)
}

/// One locked server state.
pub type SharedServerState = Arc<Mutex<ServerState>>;

/// The server states by connection; see the module header.
#[derive(Debug, Default)]
pub struct ServerRegistry {
    servers: RwLock<BTreeMap<ServerConnectionHandlerId, SharedServerState>>,
}

impl ServerRegistry {
    /// No connection.
    pub fn new() -> Self {
        Self::default()
    }

    /// The state of `server`, if the connection is established.
    pub fn get(&self, server: ServerConnectionHandlerId) -> Option<SharedServerState> {
        self.servers.read().get(&server).cloned()
    }

    /// The state of `server` without waiting: `None` while the map is being written (counted in
    /// `diagnostics` as a skipped block) or the connection is unknown. The audio callbacks use it.
    pub fn try_get(
        &self,
        server: ServerConnectionHandlerId,
        diagnostics: &AudioDiagnostics,
    ) -> Option<SharedServerState> {
        let Some(servers) = self.servers.try_read() else {
            diagnostics.record_skipped_block();
            return None;
        };
        servers.get(&server).cloned()
    }

    /// The state of `server`, created with `create` when it is new.
    pub fn get_or_insert(
        &self,
        server: ServerConnectionHandlerId,
        create: impl FnOnce() -> ServerState,
    ) -> SharedServerState {
        if let Some(state) = self.get(server) {
            return state;
        }
        let fresh = Arc::new(Mutex::new(create()));
        Arc::clone(self.servers.write().entry(server).or_insert(fresh))
    }

    /// Forgets `server`; its state drops once no callback holds it.
    pub fn remove(&self, server: ServerConnectionHandlerId) -> Option<SharedServerState> {
        self.servers.write().remove(&server)
    }

    /// Every connection with its state.
    pub fn all(&self) -> Vec<(ServerConnectionHandlerId, SharedServerState)> {
        self.servers
            .read()
            .iter()
            .map(|(server, state)| (*server, Arc::clone(state)))
            .collect()
    }

    /// Forgets every connection.
    pub fn clear(&self) {
        self.servers.write().clear();
    }
}
