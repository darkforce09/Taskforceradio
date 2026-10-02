//! The audio drivers of [`MockHost`]: one voice block through each of the plugin's voice
//! callbacks, and the custom 3D rolloff.
//!
//! **Role:** `post_process_voice`, `captured_voice`, `mixed_playback` and `custom_3d_rolloff`
//! copy the caller's samples, pass them to the export with the counts, speaker array and flags
//! TeamSpeak passes, and return what the plugin left in them.
//!
//! **Position:** an `impl MockHost` block beside `host_lifecycle`; called by tests on their own
//! thread, which stands in for TeamSpeak's audio threads.
//!
//! **Signals and state:** reads the current server from the fake state; the buffers are the
//! driver's own for the call.
//!
//! **Invariants:** a block is whole interleaved frames (`samples.len()` a multiple of
//! `channels`, `channels > 0`); channel `n` is speaker bit `1 << n`, and the fill mask starts with
//! every channel's bit set; a missing export is [`Error::MissingExport`].

use std::ffi::{c_int, c_uint};

use tfar_ts3_host::ClientId;

use super::host_lifecycle::MockHost;
use crate::{Error, Result};

#[cfg(test)]
#[path = "tests/audio_drivers.rs"]
mod tests;

/// The samples and fill mask a voice callback left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceBlockOutcome {
    /// The interleaved samples after the plugin ran.
    pub samples: Vec<i16>,
    /// The channel fill mask after the plugin ran.
    pub channel_fill_mask: u32,
}

/// The samples and `edited` flags the capture callback left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedVoiceOutcome {
    /// The interleaved samples after the plugin ran.
    pub samples: Vec<i16>,
    /// The `edited` flags after the plugin ran: bit 1 "changed", bit 2 "send".
    pub edited: i32,
}

/// The frame and channel counts of a block, as the C `int`s the callbacks take.
struct BlockShape {
    /// Samples per channel.
    frames: c_int,
    /// Interleaved channels.
    channels: c_int,
    /// One speaker bit per channel, in order.
    speakers: Vec<c_uint>,
    /// Every channel's bit.
    all_channels_mask: c_uint,
}

/// The shape of `sample_count` samples over `channels`, or [`Error::InvalidVoiceBlock`].
fn block_shape(sample_count: usize, channels: usize) -> Result<BlockShape> {
    let invalid = || Error::InvalidVoiceBlock {
        sample_count,
        channels,
    };
    if channels == 0 || !sample_count.is_multiple_of(channels) {
        return Err(invalid());
    }
    let frames = c_int::try_from(sample_count / channels).map_err(|_| invalid())?;
    let channel_count = u32::try_from(channels).map_err(|_| invalid())?;
    let speakers = (0..channel_count)
        .map(|channel| 1_u32.checked_shl(channel).unwrap_or(0))
        .collect();
    let all_channels_mask = 1_u32
        .checked_shl(channel_count)
        .map_or(u32::MAX, |next_bit| next_bit.wrapping_sub(1));
    Ok(BlockShape {
        frames,
        channels: c_int::try_from(channels).map_err(|_| invalid())?,
        speakers,
        all_channels_mask,
    })
}

impl MockHost {
    /// Runs `ts3plugin_onEditPostProcessVoiceDataEvent` for `client` on the current server over
    /// a copy of `samples` (`channels` interleaved) with every channel marked filled.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidVoiceBlock`] for a ragged block; [`Error::MissingExport`].
    pub fn post_process_voice(
        &self,
        client: ClientId,
        samples: &[i16],
        channels: usize,
    ) -> Result<VoiceBlockOutcome> {
        let shape = block_shape(samples.len(), channels)?;
        let export =
            self.exports()
                .on_edit_post_process_voice_data_event
                .ok_or(Error::MissingExport {
                    symbol: "ts3plugin_onEditPostProcessVoiceDataEvent",
                })?;
        let mut buffer = samples.to_vec();
        let mut channel_fill_mask = shape.all_channels_mask;
        // SAFETY: `buffer` holds `frames * channels` samples, `speakers` one entry per channel,
        // and the mask is a writable `unsigned int`; all are exclusive to the call.
        unsafe {
            export(
                self.current_server().0,
                client.0,
                buffer.as_mut_ptr(),
                shape.frames,
                shape.channels,
                shape.speakers.as_ptr(),
                &raw mut channel_fill_mask,
            );
        };
        Ok(VoiceBlockOutcome {
            samples: buffer,
            channel_fill_mask,
        })
    }

    /// Runs `ts3plugin_onEditCapturedVoiceDataEvent` on the current server over a copy of our
    /// microphone `samples` with the `edited` flags given.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidVoiceBlock`] for a ragged block; [`Error::MissingExport`].
    pub fn captured_voice(
        &self,
        samples: &[i16],
        channels: usize,
        edited: i32,
    ) -> Result<CapturedVoiceOutcome> {
        let shape = block_shape(samples.len(), channels)?;
        let export =
            self.exports()
                .on_edit_captured_voice_data_event
                .ok_or(Error::MissingExport {
                    symbol: "ts3plugin_onEditCapturedVoiceDataEvent",
                })?;
        let mut buffer = samples.to_vec();
        let mut edited = edited;
        // SAFETY: `buffer` holds `frames * channels` samples and `edited` is a writable `int`;
        // both are exclusive to the call.
        unsafe {
            export(
                self.current_server().0,
                buffer.as_mut_ptr(),
                shape.frames,
                shape.channels,
                &raw mut edited,
            );
        };
        Ok(CapturedVoiceOutcome {
            samples: buffer,
            edited,
        })
    }

    /// Runs `ts3plugin_onEditMixedPlaybackVoiceDataEvent` on the current server over a copy of
    /// the mixed `samples` with the fill mask given.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidVoiceBlock`] for a ragged block; [`Error::MissingExport`].
    pub fn mixed_playback(
        &self,
        samples: &[i16],
        channels: usize,
        channel_fill_mask: u32,
    ) -> Result<VoiceBlockOutcome> {
        let shape = block_shape(samples.len(), channels)?;
        let export = self
            .exports()
            .on_edit_mixed_playback_voice_data_event
            .ok_or(Error::MissingExport {
                symbol: "ts3plugin_onEditMixedPlaybackVoiceDataEvent",
            })?;
        let mut buffer = samples.to_vec();
        let mut channel_fill_mask = channel_fill_mask;
        // SAFETY: `buffer` holds `frames * channels` samples, `speakers` one entry per channel,
        // and the mask is a writable `unsigned int`; all are exclusive to the call.
        unsafe {
            export(
                self.current_server().0,
                buffer.as_mut_ptr(),
                shape.frames,
                shape.channels,
                shape.speakers.as_ptr(),
                &raw mut channel_fill_mask,
            );
        };
        Ok(VoiceBlockOutcome {
            samples: buffer,
            channel_fill_mask,
        })
    }

    /// Runs `ts3plugin_onCustom3dRolloffCalculationClientEvent` for `client` at `distance`
    /// metres, starting from full volume, and returns the volume the plugin set.
    ///
    /// # Errors
    ///
    /// [`Error::MissingExport`].
    pub fn custom_3d_rolloff(&self, client: ClientId, distance: f32) -> Result<f32> {
        let export = self
            .exports()
            .on_custom_3d_rolloff_calculation_client_event
            .ok_or(Error::MissingExport {
                symbol: "ts3plugin_onCustom3dRolloffCalculationClientEvent",
            })?;
        let mut volume = 1.0_f32;
        // SAFETY: `volume` is a writable `float` exclusive to the call.
        unsafe { export(self.current_server().0, client.0, distance, &raw mut volume) };
        Ok(volume)
    }
}
