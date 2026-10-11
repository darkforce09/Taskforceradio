//! What the plugin reads from TeamSpeak for one voice frame, and what the frame came to.

use crate::client_state::ClientId;

/// The facts of one post-process voice frame that only TeamSpeak knows. The plugin reads them
/// before it takes the server lock, so no TeamSpeak call runs under it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoiceFrameContext<'frame> {
    /// The TeamSpeak client the frame is from.
    pub sender: ClientId,
    /// The local player's TeamSpeak client id.
    pub listener: ClientId,
    /// Whether the sender's client metadata shows the plugin running, connected and in game
    /// (the metadata test of `isPluginEnabledForUser`,
    /// `reference/tfar_0_9_12/ts/src/plugin.cpp:2526`).
    pub sender_metadata_shows_plugin: bool,
    /// The name of the channel the sender sits in; empty when TeamSpeak cannot tell it.
    pub sender_channel_name: &'frame str,
    /// Whether the plugin is in a game session (`GameSession::is_in_game`).
    pub is_in_game: bool,
}

/// The facts of one captured frame of the local player's own voice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnVoiceContext<'frame> {
    /// The local player's TeamSpeak client id.
    pub my_client_id: ClientId,
    /// Whether the local player's own metadata shows the plugin running.
    pub my_metadata_shows_plugin: bool,
    /// The name of the channel the local player sits in.
    pub my_channel_name: &'frame str,
    /// Whether the plugin is in a game session.
    pub is_in_game: bool,
}

impl OwnVoiceContext<'_> {
    /// The post-process context of the local player's own voice: sender and listener are both
    /// the local player.
    pub fn as_voice_frame(&self) -> VoiceFrameContext<'_> {
        VoiceFrameContext {
            sender: self.my_client_id,
            listener: self.my_client_id,
            sender_metadata_shows_plugin: self.my_metadata_shows_plugin,
            sender_channel_name: self.my_channel_name,
            is_in_game: self.is_in_game,
        }
    }
}

/// What became of one voice frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VoiceOutcome {
    /// The frame went through the voice chain: direct speech, radio and the global volume.
    Processed,
    /// The frame was replaced by silence: a dead listener in serious mode, an alive listener
    /// in serious mode hearing a player without current data, or the local player's own frame
    /// without current data.
    Silenced,
    /// The frame is untouched: the sender is not a TFAR player in game, so they are heard as
    /// TeamSpeak delivers them.
    PassThrough,
}
