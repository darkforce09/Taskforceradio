//! The TeamSpeak mixed-playback block: its bounds, its fill mask and saturating stereo mixing into
//! it.
//!
//! **Role:** checks a block against its channel and frame counts, silences the front channels
//! TeamSpeak has not filled, and adds stereo frames into channels 0 and 1 (or the one mono
//! channel) without wrapping.
//!
//! **Position:** a module of `tfar_audio::playback_mixer`; used by
//! `PlaybackMixer::mix_into` for every playback and stream. The specification is the mixing loop of
//! `reference/tfar_1_0/ts/src/PlaybackHandler.cpp:36-79` and
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:2600-2651`, held inside `channels * frames` samples.
//!
//! **Signals and state:** none; pure functions over the caller's block, called from the TeamSpeak
//! audio thread. Nothing allocates.
//!
//! **Invariants:** nothing reads or writes past `channels * frames` samples; channels 2 and up are
//! never touched; a mono block receives the average of left and right.

use crate::playback_mixer::error::{Error, Result};

#[cfg(test)]
#[path = "tests/output_block.rs"]
mod tests;

/// The first `channels * frames` samples of `samples`, refusing a zero channel count, an
/// overflowing count and a short slice.
pub(crate) fn checked_block(
    samples: &mut [i16],
    channels: usize,
    frames: usize,
) -> Result<&mut [i16]> {
    if channels == 0 {
        return Err(Error::NoChannels);
    }
    let needed = channels
        .checked_mul(frames)
        .ok_or(Error::SampleCountOverflow { channels, frames })?;
    let actual = samples.len();
    samples
        .get_mut(..needed)
        .ok_or(Error::SampleSliceTooShort { needed, actual })
}

/// The fill-mask bits of the channels the mixer writes: channels 0 and 1, or channel 0 alone in a
/// mono block.
pub(crate) fn written_channel_bits(channels: usize) -> u32 {
    if channels >= 2 { 0b11 } else { 0b01 }
}

/// Zeroes channel 0 and channel 1 of every frame of `block` whose bit in `channel_fill_mask` is
/// clear, so the mixer adds to silence rather than to whatever the buffer held; channels whose bit
/// is set hold TeamSpeak's own mix and are kept.
pub(crate) fn silence_unfilled_channels(
    block: &mut [i16],
    channels: usize,
    channel_fill_mask: u32,
) {
    let clear_left = channel_fill_mask & 0b01 == 0;
    let clear_right = channel_fill_mask & 0b10 == 0;
    if !clear_left && !clear_right {
        return;
    }
    for frame in block.chunks_exact_mut(channels) {
        match frame {
            [left, right, ..] => {
                if clear_left {
                    *left = 0;
                }
                if clear_right {
                    *right = 0;
                }
            }
            [mono] if clear_left => *mono = 0,
            _ => {}
        }
    }
}

/// Adds interleaved stereo samples from `source` into `block`, frame by frame and saturating:
/// left into channel 0 and right into channel 1, or their average into a mono channel. Stops when
/// the block or the source runs out; returns the frames mixed.
pub(crate) fn mix_stereo_samples(
    block: &mut [i16],
    channels: usize,
    mut source: impl Iterator<Item = i16>,
) -> usize {
    let mut mixed_frames = 0_usize;
    for frame in block.chunks_exact_mut(channels) {
        let (Some(left), Some(right)) = (source.next(), source.next()) else {
            break;
        };
        match frame {
            [out_left, out_right, ..] => {
                *out_left = out_left.saturating_add(left);
                *out_right = out_right.saturating_add(right);
            }
            [out_mono] => *out_mono = out_mono.saturating_add(left.midpoint(right)),
            [] => break,
        }
        mixed_frames = mixed_frames.saturating_add(1);
    }
    mixed_frames
}
