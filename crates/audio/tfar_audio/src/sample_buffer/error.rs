//! The error type of `tfar_audio::sample_buffer`.

/// Every failure `tfar_audio::sample_buffer` reports.
///
/// Each variant carries the counts a caller needs to see which of its inputs is wrong; a TeamSpeak
/// callback turns it into a pass-through of the untouched frame.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A sample block was described with zero channels, which holds no frame at all.
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
    /// The interleaved sample slice is shorter than its channel and frame counts say.
    #[error(
        "the sample slice holds {actual} samples, but {channels} channels times {frames} frames \
         need {needed}"
    )]
    SampleSliceTooShort {
        /// The samples the counts require: `channels * frames`.
        needed: usize,
        /// The samples the slice holds.
        actual: usize,
        /// The channel count that was given.
        channels: usize,
        /// The frame count that was given.
        frames: usize,
    },
    /// The stereo buffer holds fewer frames than the block being written back.
    #[error("the stereo buffer holds {actual} frames, but {needed} frames are written back")]
    StereoBufferTooShort {
        /// The frames the write-back asks for.
        needed: usize,
        /// The frames the stereo buffer holds.
        actual: usize,
    },
    /// A planar `f32` channel slice is shorter than the stereo buffer it pairs with.
    #[error(
        "the planar {channel} slice holds {actual} samples, but the stereo buffer has {needed} frames"
    )]
    PlanarSliceTooShort {
        /// Which planar channel is short: `"left"` or `"right"`.
        channel: &'static str,
        /// The frames the stereo buffer holds.
        needed: usize,
        /// The samples the planar slice holds.
        actual: usize,
    },
}

/// The result of a fallible `tfar_audio::sample_buffer` operation.
pub type Result<T> = core::result::Result<T, Error>;
