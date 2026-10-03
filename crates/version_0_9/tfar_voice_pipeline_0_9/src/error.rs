//! The error type of `tfar_voice_pipeline_0_9`.

/// Every failure `tfar_voice_pipeline_0_9` reports. Processing a frame never fails; only handing
/// the local player's own voice to the playback mixer and building a remote push-to-talk sound
/// can.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The playback mixer refused the local player's own voice block.
    #[error("the own-voice stream refused a block: {source}")]
    OwnVoiceStream {
        /// The playback mixer's error.
        source: tfar_playback_mixer::Error,
    },
    /// A remote push-to-talk sound could not become a playback: its WAV is not whole stereo
    /// frames.
    #[error("the remote push-to-talk sound could not be built: {source}")]
    RemoteTangentSound {
        /// The playback mixer's error.
        source: tfar_playback_mixer::Error,
    },
}

/// The result of a fallible `tfar_voice_pipeline_0_9` operation.
pub type Result<T> = core::result::Result<T, Error>;
