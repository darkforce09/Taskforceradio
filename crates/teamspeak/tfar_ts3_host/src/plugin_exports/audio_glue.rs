//! The bodies of the audio exports: the post-process, mixed-playback and captured voice blocks and
//! the custom 3D rolloff.
//!
//! **Role:** turns TeamSpeak's sample pointer and counts into a bounded `&mut [i16]` and its
//! speaker and mask pointers into references, then calls the matching [`Ts3Plugin`] method.
//!
//! **Position:** called only from the expansion of [`ts3_plugin!`](crate::ts3_plugin); public but
//! hidden.
//!
//! **Signals and state:** runs on TeamSpeak's audio threads, frame by frame: it allocates nothing,
//! takes no lock and calls no TeamSpeak function.
//!
//! **Invariants:** a slice is built only from a non-null pointer and counts whose product fits in
//! `usize`; anything else becomes an empty slice with `frames == 0`. A null mask, `edited` or
//! `volume` pointer becomes a local value the plugin may write without effect.

use std::ffi::{c_int, c_uint};

use super::event_arguments::{CapturedVoiceData, MixedPlaybackVoiceData, PostProcessVoiceData};
use super::plugin_trait::Ts3Plugin;
use crate::{ClientId, ServerConnectionHandlerId};

/// The raw `short* samples, int sampleCount, int channels` of a voice callback.
#[derive(Clone, Copy)]
pub struct RawVoiceBlock {
    /// `short* samples`: interleaved, `sampleCount * channels` long.
    pub samples: *mut i16,
    /// `int sampleCount`: frames per channel.
    pub sample_count: c_int,
    /// `int channels`.
    pub channels: c_int,
}

/// The samples of `block` as a slice with its channel and frame counts.
///
/// # Safety
///
/// `block.samples` is null or points at `sample_count * channels` initialised, writable `i16`s
/// that nothing else touches for `'block`, as TeamSpeak guarantees for the callback.
unsafe fn sample_slice<'block>(block: RawVoiceBlock) -> (&'block mut [i16], usize, usize) {
    let channels = usize::try_from(block.channels).unwrap_or(0);
    let frames = usize::try_from(block.sample_count).unwrap_or(0);
    match channels.checked_mul(frames) {
        Some(length) if !block.samples.is_null() && length > 0 => {
            // SAFETY: non-null, and `length` is exactly TeamSpeak's `sampleCount * channels`, the
            // element count the caller's contract makes valid and exclusive for `'block`.
            let samples = unsafe { std::slice::from_raw_parts_mut(block.samples, length) };
            (samples, channels, frames)
        }
        _ => (&mut [], channels, 0),
    }
}

/// The speaker array of a block: one `c_uint` per channel, or empty when TeamSpeak passes null.
///
/// # Safety
///
/// `speakers` is null or points at `channels` readable `c_uint`s valid for `'block`.
unsafe fn speaker_slice<'block>(speakers: *const c_uint, channels: usize) -> &'block [u32] {
    if speakers.is_null() || channels == 0 {
        return &[];
    }
    // SAFETY: non-null and `channels` elements long for `'block` by the caller's contract.
    unsafe { std::slice::from_raw_parts(speakers, channels) }
}

/// `onEditPostProcessVoiceDataEvent`.
///
/// # Safety
///
/// `block` follows [`RawVoiceBlock`]'s contract; `speakers` is null or holds one `c_uint` per
/// channel; `fill_mask` is null or a writable `c_uint`; all valid for this call.
pub unsafe fn post_process_voice_data<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    client: u16,
    block: RawVoiceBlock,
    speakers: *const c_uint,
    fill_mask: *mut c_uint,
) {
    // SAFETY: the caller's contract covers the sample block for this call.
    let (samples, channels, frames) = unsafe { sample_slice(block) };
    // SAFETY: the caller's contract covers the speaker array.
    let channel_speakers = unsafe { speaker_slice(speakers, channels) };
    let mut unused_mask = 0;
    // SAFETY: `fill_mask` is null or a writable `c_uint` exclusive to this call.
    let channel_fill_mask = unsafe { fill_mask.as_mut() }.unwrap_or(&mut unused_mask);
    plugin.on_edit_post_process_voice_data(PostProcessVoiceData {
        server: ServerConnectionHandlerId(server),
        client: ClientId(client),
        samples,
        channels,
        frames,
        channel_speakers,
        channel_fill_mask,
    });
}

/// `onEditMixedPlaybackVoiceDataEvent`.
///
/// # Safety
///
/// As [`post_process_voice_data`].
pub unsafe fn mixed_playback_voice_data<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    block: RawVoiceBlock,
    speakers: *const c_uint,
    fill_mask: *mut c_uint,
) {
    // SAFETY: the caller's contract covers the sample block for this call.
    let (samples, channels, frames) = unsafe { sample_slice(block) };
    // SAFETY: the caller's contract covers the speaker array.
    let channel_speakers = unsafe { speaker_slice(speakers, channels) };
    let mut unused_mask = 0;
    // SAFETY: `fill_mask` is null or a writable `c_uint` exclusive to this call.
    let channel_fill_mask = unsafe { fill_mask.as_mut() }.unwrap_or(&mut unused_mask);
    plugin.on_edit_mixed_playback_voice_data(MixedPlaybackVoiceData {
        server: ServerConnectionHandlerId(server),
        samples,
        channels,
        frames,
        channel_speakers,
        channel_fill_mask,
    });
}

/// `onEditCapturedVoiceDataEvent`.
///
/// # Safety
///
/// `block` follows [`RawVoiceBlock`]'s contract; `edited` is null or a writable `int`; both valid
/// for this call.
pub unsafe fn captured_voice_data<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    block: RawVoiceBlock,
    edited: *mut c_int,
) {
    // SAFETY: the caller's contract covers the sample block for this call.
    let (samples, channels, frames) = unsafe { sample_slice(block) };
    let mut unused_edited = 0;
    // SAFETY: `edited` is null or a writable `int` exclusive to this call.
    let edited = unsafe { edited.as_mut() }.unwrap_or(&mut unused_edited);
    plugin.on_edit_captured_voice_data(CapturedVoiceData {
        server: ServerConnectionHandlerId(server),
        samples,
        channels,
        frames,
        edited,
    });
}

/// `onCustom3dRolloffCalculationClientEvent`.
///
/// # Safety
///
/// `volume` is null or a writable `float` valid for this call.
pub unsafe fn custom_3d_rolloff_calculation_client<P: Ts3Plugin>(
    plugin: &'static P,
    server: u64,
    client: u16,
    distance: f32,
    volume: *mut f32,
) {
    let mut unused_volume = 1.0;
    // SAFETY: `volume` is null or a writable `float` exclusive to this call.
    let volume = unsafe { volume.as_mut() }.unwrap_or(&mut unused_volume);
    plugin.on_custom_3d_rolloff_calculation_client(
        ServerConnectionHandlerId(server),
        ClientId(client),
        distance,
        volume,
    );
}
