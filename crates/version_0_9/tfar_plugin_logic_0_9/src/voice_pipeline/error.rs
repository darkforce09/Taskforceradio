//! The error type of `tfar_plugin_logic_0_9::voice_pipeline`.

/// Every failure `tfar_plugin_logic_0_9::voice_pipeline` reports. Processing a frame never fails; only handing
/// the local player's own voice to the playback mixer and building a remote push-to-talk sound
/// can.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The playback mixer refused the local player's own voice block.
    #[error("the own-voice stream refused a block: {source}")]
    OwnVoiceStream {
        /// The playback mixer's error.
        source: tfar_audio::playback_mixer::Error,
    },
    /// A remote push-to-talk sound could not become a playback: its WAV is not whole stereo
    /// frames.
    #[error("the remote push-to-talk sound could not be built: {source}")]
    RemoteTangentSound {
        /// The playback mixer's error.
        source: tfar_audio::playback_mixer::Error,
    },
}

/// The result of a fallible `tfar_plugin_logic_0_9::voice_pipeline` operation.
pub type Result<T> = core::result::Result<T, Error>;
