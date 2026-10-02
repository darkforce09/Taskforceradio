//! The error type of `tfar_playback_mixer`.
//!
//! **Role:** names every failure of the crate: a WAV file that cannot be found, read or played, a
//! sample count that is not whole stereo frames, a full queue or stream table, and a TeamSpeak
//! block whose counts do not describe its samples.
//!
//! **Position:** a module of `tfar_playback_mixer`, tier 1 of crates/audio; returned by every
//! fallible function of the crate and logged by the plugins.
//!
//! **Signals and state:** none; plain values, built on whichever thread failed.
//!
//! **Invariants:** every WAV error carries the path of its file; every block error carries the
//! counts the caller passed.

use std::path::PathBuf;

use crate::playback::PlaybackId;

/// Every failure `tfar_playback_mixer` reports.
///
/// A WAV error names the file; a block error carries the counts the caller passed. The mixed
/// playback callback turns a block error into a pass-through of the untouched frame.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A WAV path could not be resolved to a canonical path (the file is missing or unreadable).
    #[error("the radio sound {} cannot be found: {source}", path.display())]
    WavePath {
        /// The path that was given.
        path: PathBuf,
        /// What the file system reported.
        source: std::io::Error,
    },
    /// `hound` could not open or decode a WAV file.
    #[error("the radio sound {} cannot be read as WAV: {source}", path.display())]
    WaveRead {
        /// The canonical path of the file.
        path: PathBuf,
        /// What `hound` reported.
        source: hound::Error,
    },
    /// A WAV file is not 48 kHz, 2-channel, 16-bit integer PCM, the only format the mixer plays.
    #[error(
        "the radio sound {} is {sample_rate} Hz, {channels} channels, {bits_per_sample}-bit \
         {sample_format:?}, but only 48000 Hz, 2 channels, 16-bit Int is played",
        path.display()
    )]
    UnsupportedWaveFormat {
        /// The canonical path of the file.
        path: PathBuf,
        /// The file's sample rate in Hz.
        sample_rate: u32,
        /// The file's channel count.
        channels: u16,
        /// The file's bits per sample.
        bits_per_sample: u16,
        /// Whether the file holds integer or float samples.
        sample_format: hound::SampleFormat,
    },
    /// A WAV file ends inside a frame: its sample count is odd for two channels.
    #[error(
        "the radio sound {} holds {samples} samples, which is not whole stereo frames",
        path.display()
    )]
    PartialWaveFrame {
        /// The canonical path of the file.
        path: PathBuf,
        /// The samples the file holds.
        samples: usize,
    },
    /// Interleaved stereo samples were given in an odd count, so the last frame is cut.
    #[error("{samples} interleaved stereo samples are not whole frames")]
    OddStereoSampleCount {
        /// The samples that were given.
        samples: usize,
    },
    /// The queue to the audio thread is full; the playback was dropped.
    #[error("the playback queue is full, so {id} was dropped")]
    PlaybackQueueFull {
        /// The playback that did not fit.
        id: PlaybackId,
    },
    /// Every stream slot holds another name already.
    #[error("the mixer holds at most {maximum} streams, so the stream `{name}` cannot be added")]
    TooManyStreams {
        /// The stream name that did not fit.
        name: String,
        /// The most streams the mixer holds.
        maximum: usize,
    },
    /// A sample block was described with zero channels.
    #[error("a sample block needs at least one channel, but 0 channels were given")]
    NoChannels,
    /// The channel count times the frame count does not fit in `usize`.
    #[error("{channels} channels times {frames} frames overflows the sample count")]
    SampleCountOverflow {
        /// The channel count that was given.
        channels: usize,
        /// The frame count that was given.
        frames: usize,
    },
    /// The sample slice is shorter than its channel and frame counts say.
    #[error("the sample slice holds {actual} samples, but the block needs {needed}")]
    SampleSliceTooShort {
        /// The samples the counts require: `channels * frames`.
        needed: usize,
        /// The samples the slice holds.
        actual: usize,
    },
}

/// The result of a fallible `tfar_playback_mixer` operation.
pub type Result<T> = core::result::Result<T, Error>;
