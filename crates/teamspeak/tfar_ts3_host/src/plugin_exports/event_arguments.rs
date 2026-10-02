//! The argument bundles of the [`Ts3Plugin`](crate::Ts3Plugin) callbacks: who caused an event,
//! a client move, a server error, a plugin command and the three voice-data blocks.
//!
//! **Role:** the safe shapes the export glue builds from TeamSpeak's raw arguments, so a plugin
//! sees IDs as newtypes, text as `&str` and samples as bounded slices.
//!
//! **Position:** inside `plugin_exports`; built by `event_glue` and `audio_glue`, read by plugins.
//!
//! **Signals and state:** borrowed values that live for one callback; the voice bundles are built
//! on TeamSpeak's audio threads.
//!
//! **Invariants:** in every voice bundle `samples.len() == channels * frames`; a null or
//! inconsistent TeamSpeak buffer arrives as an empty slice with `frames == 0`.

use crate::{ChannelId, ClientId, ServerConnectionHandlerId, Visibility};

/// The client that caused an event: mover, kicker, editor or command sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Invoker<'event> {
    /// The invoker's client ID; `0` for the server itself.
    pub id: ClientId,
    /// The invoker's nickname.
    pub name: &'event str,
    /// The invoker's unique identity.
    pub unique_identifier: &'event str,
}

/// A client changing channel: the shared arguments of the move, timeout, kick and ban events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientMove<'event> {
    /// The server connection the move happened on.
    pub server: ServerConnectionHandlerId,
    /// The client that moved.
    pub client: ClientId,
    /// The channel it left; `0` when it joined the server.
    pub old_channel: ChannelId,
    /// The channel it entered; `0` when it left the server.
    pub new_channel: ChannelId,
    /// Whether it enters, stays in or leaves our view.
    pub visibility: Visibility,
    /// The move, timeout, kick or ban message.
    pub message: &'event str,
}

/// The arguments of `onServerErrorEvent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerError<'event> {
    /// The server connection the error belongs to.
    pub server: ServerConnectionHandlerId,
    /// TeamSpeak's error text.
    pub message: &'event str,
    /// The TeamSpeak error code.
    pub error: u32,
    /// The return code the failed request carried; empty when it carried none.
    pub return_code: &'event str,
    /// TeamSpeak's extra text.
    pub extra_message: &'event str,
}

/// The arguments of `onServerPermissionErrorEvent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerPermissionError<'event> {
    /// The server connection the error belongs to.
    pub server: ServerConnectionHandlerId,
    /// TeamSpeak's error text.
    pub message: &'event str,
    /// The TeamSpeak error code.
    pub error: u32,
    /// The return code the failed request carried; empty when it carried none.
    pub return_code: &'event str,
    /// The permission that was missing.
    pub failed_permission_id: u32,
}

/// The arguments of `onPluginCommandEvent` in its API 26 form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginCommand<'event> {
    /// The server connection the command arrived on.
    pub server: ServerConnectionHandlerId,
    /// The name of the sending plugin.
    pub plugin_name: &'event str,
    /// The command text.
    pub command: &'event str,
    /// The client that sent it.
    pub invoker: Invoker<'event>,
}

/// One block of a speaking client's voice after TeamSpeak's own processing
/// (`onEditPostProcessVoiceDataEvent`).
#[derive(Debug, PartialEq, Eq)]
pub struct PostProcessVoiceData<'block> {
    /// The server connection the voice belongs to.
    pub server: ServerConnectionHandlerId,
    /// The speaking client.
    pub client: ClientId,
    /// Interleaved samples, `channels * frames` long.
    pub samples: &'block mut [i16],
    /// Channels per frame.
    pub channels: usize,
    /// Frames in the block (TeamSpeak's `sampleCount`).
    pub frames: usize,
    /// The speaker of each channel (`SPEAKER_*` bits); empty when TeamSpeak passes none.
    pub channel_speakers: &'block [u32],
    /// Which channels hold sound: bit `n` set means channel `n` was filled.
    pub channel_fill_mask: &'block mut u32,
}

/// One block of the final mix about to reach the playback device
/// (`onEditMixedPlaybackVoiceDataEvent`).
#[derive(Debug, PartialEq, Eq)]
pub struct MixedPlaybackVoiceData<'block> {
    /// The server connection the mix belongs to.
    pub server: ServerConnectionHandlerId,
    /// Interleaved samples, `channels * frames` long.
    pub samples: &'block mut [i16],
    /// Channels per frame.
    pub channels: usize,
    /// Frames in the block (TeamSpeak's `sampleCount`).
    pub frames: usize,
    /// The speaker of each channel (`SPEAKER_*` bits); empty when TeamSpeak passes none.
    pub channel_speakers: &'block [u32],
    /// Which channels hold sound: bit `n` set means channel `n` was filled.
    pub channel_fill_mask: &'block mut u32,
}

/// One block of our own microphone (`onEditCapturedVoiceDataEvent`).
#[derive(Debug, PartialEq, Eq)]
pub struct CapturedVoiceData<'block> {
    /// The server connection the capture belongs to.
    pub server: ServerConnectionHandlerId,
    /// Interleaved samples, `channels * frames` long.
    pub samples: &'block mut [i16],
    /// Channels per frame.
    pub channels: usize,
    /// Frames in the block (TeamSpeak's `sampleCount`).
    pub frames: usize,
    /// TeamSpeak's `edited` flags: the value `1` marks the block as changed by a plugin, the value
    /// `2` is set while the block is going to be sent.
    pub edited: &'block mut i32,
}
