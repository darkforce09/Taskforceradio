//! `VoiceFrameContext` and `VoiceOutcome`: what one voice frame is processed with, and how.

use std::time::Instant;

use tfar_antennas_1_0::AntennaManager;
use tfar_plugin_settings_1_0::PluginSettings;

/// What the plugin app hands the voice processor with one frame: the settings, the radio
/// towers, the TeamSpeak-side answers the pipeline cannot read itself, and the time.
///
/// The app fills it under its server lock: `is_in_game` from its game link state,
/// `is_serious_mode` from its serious channel cache (`isSeriousModeEnabled`: the sender is in
/// the serious channel), and `sender_metadata` only when
/// `SessionState::needs_plugin_metadata` says the 10 s cache has expired, read with the lock
/// released.
#[derive(Clone, Copy, Debug)]
pub struct VoiceFrameContext<'a> {
    /// The plugin settings (`voiceCone`, `intercomVolume`, the spectator rules, …).
    pub settings: &'a PluginSettings,
    /// The radio towers.
    pub antennas: &'a AntennaManager,
    /// A game is running (`getCurrentlyInGame`); otherwise every frame passes through.
    pub is_in_game: bool,
    /// Serious mode is on for the sender: the serious channel is set and the sender is in it.
    pub is_serious_mode: bool,
    /// The sender's TeamSpeak metadata, read because the plugin-enabled cache expired; `None`
    /// when the cache still stands or the metadata is empty.
    pub sender_metadata: Option<&'a str>,
    /// The time positions are extrapolated to and liveness is judged at.
    pub now: Instant,
}

/// What [`crate::VoiceProcessor::process_sender`] did with a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoiceOutcome {
    /// The frame went through the TFAR chain (direct speech, radios, global volume).
    Processed,
    /// The frame is silence: the sender is the local player without TFAR, or a spectator or
    /// serious-mode rule mutes it.
    Silenced,
    /// The frame is untouched: no game runs, the sender is unknown, or the sender (or the local
    /// player) is not in a TFAR game, so they are heard as plain TeamSpeak voice.
    PassThrough,
}
