//! `ServerConnection`: what the plugin keeps per TeamSpeak server connection, behind one lock.
//!
//! **Role:** hold the 0.9 client state, the routing `MuteTracker`, the voice processor with its
//! stereo scratch, the TeamSpeak facts of the channel and the serious-mode return channel in one
//! `ServerState` behind one `parking_lot::Mutex`, beside the connection's playback mixer; and
//! make the sound state the senders' voices need (`ensure_dsp` over `dsp_requirements`).
//!
//! **Position:** made on `STATUS_CONNECTION_ESTABLISHED` (or at init for a connection already
//! up) and dropped on `STATUS_DISCONNECTED` by `PluginCore`; every command path and the voice
//! callbacks reach it through the core's server map.
//!
//! **Signals and state:** command threads (TeamSpeak's main thread, the game link thread, the
//! service thread) `lock` the state for a short decision and call TeamSpeak only after
//! releasing it; the audio threads only `try_lock` it and pass the frame through on contention.
//! The mixer sits outside the lock: the mixed-playback callback never touches the state.
//!
//! **Invariants:** sound objects are made here, on command threads, never on the audio thread;
//! the stereo scratch is sized once for `DEFAULT_MAX_FRAMES`, so a TeamSpeak block within it
//! allocates nothing.

use parking_lot::Mutex;
use tfar_client_state_0_9::{ClientId as StateClientId, ServerRadioData};
use tfar_hrtf::HrirLibrary;
use tfar_playback_mixer::{PlaybackMixer, StreamProducer};
use tfar_radio_routing_0_9::{MuteTracker, dsp_requirements};
use tfar_sample_buffer::StereoBuffer;
use tfar_ts3_host::{ChannelId, ServerConnectionHandlerId};
use tfar_voice_pipeline_0_9::{DEFAULT_MAX_FRAMES, OWN_VOICE_STREAM_NAME, VoiceProcessor};

use crate::channel_snapshot::{ChannelSnapshot, TeamSpeakFacts};

#[cfg(test)]
#[path = "tests/server_connection.rs"]
mod tests;

/// Everything the plugin decides on for one server connection; see the module header.
#[derive(Debug)]
pub struct ServerState {
    /// The 0.9 radio state: players, radios, frames, the local player's settings.
    pub radio: ServerRadioData,
    /// The clients this plugin muted.
    pub mutes: MuteTracker,
    /// The voice chain with its scratch.
    pub voice: VoiceProcessor,
    /// What the voice callbacks know from TeamSpeak.
    pub facts: TeamSpeakFacts,
    /// The stereo copy of a post-process block.
    pub voice_scratch: StereoBuffer,
    /// The stereo copy of a captured block.
    pub captured_scratch: StereoBuffer,
    /// The writing end of the mixer's own-voice stream, which the captured-voice callback pushes
    /// into without blocking; `None` when the mixer had no stream slot left.
    pub own_voice_stream: Option<StreamProducer>,
    /// The channel the local player sat in before serious mode moved them; `None` when serious
    /// mode moved nobody.
    pub channel_before_serious_mode: Option<ChannelId>,
    /// Whether the running game started before the mission named its serious channel, so the
    /// first `VERSION` that names one still makes the move.
    pub is_serious_move_pending: bool,
    /// The last voice outcome logged per TeamSpeak client id (0 none, 1 processed, 2 silenced,
    /// 3 passed through), so the voice callback logs only changes; sized for every `anyID`.
    pub logged_voice_outcomes: Box<[u8]>,
}

impl ServerState {
    /// The state of a fresh connection on which the local player is `my_nickname`.
    /// Allocates the scratch: call it off the audio thread.
    pub fn new(my_nickname: String) -> Self {
        Self {
            radio: ServerRadioData::new(my_nickname),
            mutes: MuteTracker::new(),
            voice: VoiceProcessor::new(DEFAULT_MAX_FRAMES),
            facts: TeamSpeakFacts::default(),
            voice_scratch: StereoBuffer::with_frames(DEFAULT_MAX_FRAMES),
            captured_scratch: StereoBuffer::with_frames(DEFAULT_MAX_FRAMES),
            own_voice_stream: None,
            channel_before_serious_mode: None,
            is_serious_move_pending: false,
            logged_voice_outcomes: vec![0; usize::from(u16::MAX) + 1].into_boxed_slice(),
        }
    }

    /// Takes in a channel snapshot: the nickname list, our nickname and the voice facts.
    pub fn apply_channel_snapshot(&mut self, snapshot: &ChannelSnapshot) {
        let pairs = snapshot.nickname_pairs();
        for (client_id, nickname) in &pairs {
            match self.radio.client(nickname).map(|client| client.client_id) {
                None => tracing::info!(
                    nickname = %nickname,
                    client = client_id.0,
                    "nickname list: a new client entry"
                ),
                Some(previous) if previous != *client_id => tracing::info!(
                    nickname = %nickname,
                    previous = previous.0,
                    client = client_id.0,
                    "nickname list: a nickname moved to another client id"
                ),
                Some(_) => {}
            }
        }
        self.radio.update_nicknames(&pairs);
        if !snapshot.my_nickname.is_empty() {
            self.radio.set_my_nickname(&snapshot.my_nickname);
        }
        self.facts.replace_with(snapshot);
    }

    /// Makes the sound state the voice of `nickname` needs for the local player to hear it, as
    /// `tfar_radio_routing_0_9::dsp_requirements` plans it. A sender without current data needs
    /// none; a design the filter library refuses is logged.
    pub fn ensure_sender_dsp(&mut self, nickname: &str, hrir: Option<&HrirLibrary>) {
        let Some(listener) = self.facts.my_client else {
            return;
        };
        let Ok(requirements) = dsp_requirements(&self.radio, nickname, listener) else {
            return;
        };
        if let Some(sender) = self.radio.client_mut(nickname)
            && let Err(error) = sender.ensure_dsp(&requirements, hrir)
        {
            tracing::warn!(%error, nickname, "the sound state of a sender could not be made");
        }
    }

    /// [`ServerState::ensure_sender_dsp`] for every known player, after a command that changes
    /// every route (`FREQ`, `SPEAKERS`, the local player's own `POS` or push-to-talk).
    pub fn ensure_every_sender_dsp(&mut self, hrir: Option<&HrirLibrary>) {
        let nicknames: Vec<String> = self.radio.clients.keys().cloned().collect();
        for nickname in nicknames {
            self.ensure_sender_dsp(&nickname, hrir);
        }
    }

    /// Clears the HRTF direction sources of every entry of `client` (direct speech and each
    /// radio), so the tail of their last utterance never replays when they start talking again;
    /// the sources stay made. Returns how many sources were reset.
    pub fn reset_direction_sources_of(&mut self, client: StateClientId) -> usize {
        let mut reset = 0;
        for sender in self
            .radio
            .clients
            .values_mut()
            .filter(|sender| sender.client_id == client)
        {
            if let Some(source) = sender.dsp.direct_voice_mut().hrtf_source_mut() {
                source.reset();
                reset += 1;
            }
            let radio_ids: Vec<String> = sender.dsp.radio_ids().map(str::to_owned).collect();
            for radio_id in &radio_ids {
                if let Some(source) = sender
                    .dsp
                    .radio_mut(radio_id)
                    .and_then(|radio| radio.hrtf_source_mut())
                {
                    source.reset();
                    reset += 1;
                }
            }
        }
        reset
    }

    /// Our own client id on the connection, as the state names it.
    pub fn my_client(&self) -> Option<StateClientId> {
        self.facts.my_client
    }
}

/// One TeamSpeak server connection: its state behind one lock, and its playback mixer.
#[derive(Debug)]
pub struct ServerConnection {
    /// The connection's id.
    pub server: ServerConnectionHandlerId,
    /// The state, locked by command threads and `try_lock`ed by the voice callbacks.
    pub state: Mutex<ServerState>,
    /// The remote radio sounds and the own-voice stream, mixed into the playback.
    pub mixer: PlaybackMixer,
}

impl ServerConnection {
    /// A connection `server` on which the local player is `my_nickname`, with the mixer's
    /// own-voice stream opened for the captured-voice callback. Allocates: call it off the audio
    /// thread.
    pub fn new(server: ServerConnectionHandlerId, my_nickname: String) -> Self {
        let mixer = PlaybackMixer::new();
        let mut state = ServerState::new(my_nickname);
        state.own_voice_stream = mixer
            .stream_producer(OWN_VOICE_STREAM_NAME)
            .map_err(|error| tracing::warn!(%error, "the own-voice stream could not be opened"))
            .ok();
        Self {
            server,
            state: Mutex::new(state),
            mixer,
        }
    }
}
