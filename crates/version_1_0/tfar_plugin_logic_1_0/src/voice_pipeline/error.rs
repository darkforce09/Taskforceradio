//! The error type of `tfar_plugin_logic_1_0::voice_pipeline`.

/// Every failure `tfar_plugin_logic_1_0::voice_pipeline` reports. The audio path itself never fails: a missing
/// DSP object skips its step. Only the DSP creation, the stream append and building a remote
/// tangent sound report errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A sender's DSP object could not be created.
    #[error("a voice DSP object could not be created: {0}")]
    Dsp(#[from] crate::client_state::Error),
    /// The playback mixer refused the local player's radio voice.
    #[error("the radio voice could not be appended to the playback mixer: {0}")]
    Mixer(#[from] tfar_audio::playback_mixer::Error),
    /// A remote tangent sound could not become a playback: its WAV is not whole stereo frames.
    #[error("the remote tangent sound could not be built: {source}")]
    RemoteTangentSound {
        /// The playback mixer's error.
        source: tfar_audio::playback_mixer::Error,
    },
}

/// The result of a fallible `tfar_plugin_logic_1_0::voice_pipeline` operation.
pub type Result<T> = core::result::Result<T, Error>;
