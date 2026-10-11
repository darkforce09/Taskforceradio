//! Bounds-checked adapters between TeamSpeak's interleaved blocks of any channel count and the
//! stereo buffer the pipelines process.
//!
//! **Role:** copies a TeamSpeak sample block (mono, stereo or more channels) into a
//! [`StereoBuffer`] and writes the processed stereo back into the block.
//!
//! **Position:** in `crates/audio`; called by the voice pipelines inside the TeamSpeak voice
//! callbacks (`onEditPostProcessVoiceDataEvent`, `onEditCapturedVoiceDataEvent`). The specification
//! is `reference/tfar_1_0/ts/src/plugin.cpp` (the post-process and captured-voice adapters) and
//! `reference/tfar_0_9_12/ts/src/plugin.cpp` (`ts3plugin_onEditPostProcessVoiceDataEvent`), with
//! every access held inside `channels * frames` samples.
//!
//! **Signals and state:** none of its own; writes the caller's [`StereoBuffer`] and sample slice,
//! from the TeamSpeak audio thread. Nothing allocates once the stereo buffer has capacity for the
//! block.
//!
//! **Invariants:** no function reads or writes past `channels * frames` samples of the TeamSpeak
//! slice; a zero channel count, an overflowing count or a short slice is an [`Error`] and leaves
//! every buffer untouched; mono becomes `[m, m]` and comes back as the average of left and right;
//! with two or more channels, channels 0 and 1 are left and right and channels 2 and up are written
//! back as silence.

use crate::sample_buffer::error::{Error, Result};
use crate::sample_buffer::stereo_buffer::StereoBuffer;

#[cfg(test)]
#[path = "tests/channel_views.rs"]
mod tests;

/// Copies `frames` frames of the interleaved `samples`, `channels` per frame, into `out` as stereo,
/// setting `out` to `frames` frames.
///
/// Mono is duplicated into both channels; with two or more channels, channel 0 becomes left and
/// channel 1 right, and the others are dropped. Samples beyond `channels * frames` are ignored.
///
/// # Errors
///
/// [`Error::NoChannels`] for `channels == 0`, [`Error::SampleCountOverflow`] when
/// `channels * frames` overflows, and [`Error::SampleSliceTooShort`] when `samples` holds fewer
/// than `channels * frames` samples. `out` is untouched on error.
pub fn copy_to_stereo(
    samples: &[i16],
    channels: usize,
    frames: usize,
    out: &mut StereoBuffer,
) -> Result<()> {
    let needed = interleaved_sample_count(channels, frames)?;
    let block = samples
        .get(..needed)
        .ok_or_else(|| slice_too_short(needed, samples.len(), channels, frames))?;
    out.set_frames(frames);
    for (stereo_frame, source_frame) in out
        .frame_pairs_mut()
        .iter_mut()
        .zip(block.chunks_exact(channels))
    {
        *stereo_frame = match source_frame {
            [left, right, ..] => [*left, *right],
            [mono] => [*mono, *mono],
            [] => continue,
        };
    }
    Ok(())
}

/// Writes the first `frames` frames of `stereo` back into the interleaved `samples`, `channels`
/// per frame.
///
/// Mono receives the average of left and right, rounded toward zero. The mono sample was
/// duplicated into both channels on the way in, so where processing kept the channels equal the
/// average returns the processed sample exactly; where processing placed the voice to one side (the
/// HRTF, a radio on one ear), the average keeps both ears at half level instead of dropping one of
/// them, and unlike a sum it never leaves the `i16` range. With two or more channels, channels 0
/// and 1 receive left and right and channels 2 and up are zeroed, as the originals' post-process
/// adapter does. Samples beyond `channels * frames` are untouched.
///
/// # Errors
///
/// [`Error::NoChannels`], [`Error::SampleCountOverflow`] and [`Error::SampleSliceTooShort`] as for
/// [`copy_to_stereo`], and [`Error::StereoBufferTooShort`] when `stereo` holds fewer than
/// `frames` frames. `samples` is untouched on error.
pub fn write_back_from_stereo(
    stereo: &StereoBuffer,
    samples: &mut [i16],
    channels: usize,
    frames: usize,
) -> Result<()> {
    let needed = interleaved_sample_count(channels, frames)?;
    if stereo.frames() < frames {
        return Err(Error::StereoBufferTooShort {
            needed: frames,
            actual: stereo.frames(),
        });
    }
    let actual = samples.len();
    let block = samples
        .get_mut(..needed)
        .ok_or_else(|| slice_too_short(needed, actual, channels, frames))?;
    for (target_frame, [left, right]) in block.chunks_exact_mut(channels).zip(stereo.frame_pairs())
    {
        match target_frame {
            [target_left, target_right, surround @ ..] => {
                *target_left = *left;
                *target_right = *right;
                surround.fill(0);
            }
            [mono] => *mono = left.midpoint(*right),
            [] => {}
        }
    }
    Ok(())
}

/// Copies the mono block `mono` into `out` as stereo, each sample into both channels, setting `out`
/// to `mono.len()` frames. The captured-voice path uses it for the player's own microphone.
pub fn mono_to_stereo(mono: &[i16], out: &mut StereoBuffer) {
    out.set_frames(mono.len());
    for (stereo_frame, sample) in out.frame_pairs_mut().iter_mut().zip(mono) {
        *stereo_frame = [*sample, *sample];
    }
}

/// `channels * frames`, refusing a zero channel count and an overflow.
fn interleaved_sample_count(channels: usize, frames: usize) -> Result<usize> {
    if channels == 0 {
        return Err(Error::NoChannels);
    }
    channels
        .checked_mul(frames)
        .ok_or(Error::SampleCountOverflow { channels, frames })
}

/// The error for a sample slice of `actual` samples where `needed` are required.
fn slice_too_short(needed: usize, actual: usize, channels: usize, frames: usize) -> Error {
    Error::SampleSliceTooShort {
        needed,
        actual,
        channels,
        frames,
    }
}
