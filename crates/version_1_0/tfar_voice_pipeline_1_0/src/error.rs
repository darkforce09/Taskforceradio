//! The error type of `tfar_voice_pipeline_1_0`.

/// Every failure `tfar_voice_pipeline_1_0` reports. The audio path itself never fails: a missing
/// DSP object skips its step. Only the DSP creation and the stream append report errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A sender's DSP object could not be created.
    #[error("a voice DSP object could not be created: {0}")]
    Dsp(#[from] tfar_client_state_1_0::Error),
    /// The playback mixer refused the local player's radio voice.
    #[error("the radio voice could not be appended to the playback mixer: {0}")]
    Mixer(#[from] tfar_playback_mixer::Error),
}

/// The result of a fallible `tfar_voice_pipeline_1_0` operation.
pub type Result<T> = core::result::Result<T, Error>;
