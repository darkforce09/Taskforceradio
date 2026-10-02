//! The audio half of [`ts3_plugin!`](crate::ts3_plugin): the voice-data and rolloff exports,
//! forwarding to `$crate::export_glue`.
//!
//! **Role:** the `onEditPostProcessVoiceDataEvent`, `onEditMixedPlaybackVoiceDataEvent`,
//! `onEditCapturedVoiceDataEvent` and `onCustom3dRolloffCalculationClientEvent` exports, with the
//! argument lists of `reference/tfar_1_0/ts/src/plugin.h` at API 26.
//!
//! **Position:** expanded only by [`ts3_plugin!`](crate::ts3_plugin); hidden from the docs.
//!
//! **Signals and state:** TeamSpeak calls these on its audio threads, frame by frame; no state.
//!
//! **Invariants:** the shells only bundle the raw arguments; the glue builds bounded slices
//! without allocating, locking or calling TeamSpeak.

/// Expands to the audio exports of [`ts3_plugin!`](crate::ts3_plugin); not for direct use.
#[doc(hidden)]
#[macro_export]
macro_rules! __ts3_plugin_audio {
    ($plugin:path) => {
        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onEditPostProcessVoiceDataEvent(
            server: u64,
            client: u16,
            samples: *mut i16,
            sample_count: ::std::ffi::c_int,
            channels: ::std::ffi::c_int,
            channel_speaker_array: *const ::std::ffi::c_uint,
            channel_fill_mask: *mut ::std::ffi::c_uint,
        ) {
            let block = $crate::export_glue::RawVoiceBlock {
                samples,
                sample_count,
                channels,
            };
            // SAFETY: TeamSpeak passes `sample_count * channels` samples, one speaker per channel
            // and a writable fill mask, all exclusive to this call (or null).
            unsafe {
                $crate::export_glue::post_process_voice_data(
                    &$plugin,
                    server,
                    client,
                    block,
                    channel_speaker_array,
                    channel_fill_mask,
                )
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onEditMixedPlaybackVoiceDataEvent(
            server: u64,
            samples: *mut i16,
            sample_count: ::std::ffi::c_int,
            channels: ::std::ffi::c_int,
            channel_speaker_array: *const ::std::ffi::c_uint,
            channel_fill_mask: *mut ::std::ffi::c_uint,
        ) {
            let block = $crate::export_glue::RawVoiceBlock {
                samples,
                sample_count,
                channels,
            };
            // SAFETY: TeamSpeak passes `sample_count * channels` samples, one speaker per channel
            // and a writable fill mask, all exclusive to this call (or null).
            unsafe {
                $crate::export_glue::mixed_playback_voice_data(
                    &$plugin,
                    server,
                    block,
                    channel_speaker_array,
                    channel_fill_mask,
                )
            }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onEditCapturedVoiceDataEvent(
            server: u64,
            samples: *mut i16,
            sample_count: ::std::ffi::c_int,
            channels: ::std::ffi::c_int,
            edited: *mut ::std::ffi::c_int,
        ) {
            let block = $crate::export_glue::RawVoiceBlock {
                samples,
                sample_count,
                channels,
            };
            // SAFETY: TeamSpeak passes `sample_count * channels` samples and a writable `edited`
            // flag, both exclusive to this call (or null).
            unsafe { $crate::export_glue::captured_voice_data(&$plugin, server, block, edited) }
        }

        #[unsafe(no_mangle)]
        #[allow(non_snake_case, reason = "the SDK export name")]
        extern "C" fn ts3plugin_onCustom3dRolloffCalculationClientEvent(
            server: u64,
            client: u16,
            distance: f32,
            volume: *mut f32,
        ) {
            // SAFETY: TeamSpeak passes a writable `volume` exclusive to this call (or null).
            unsafe {
                $crate::export_glue::custom_3d_rolloff_calculation_client(
                    &$plugin, server, client, distance, volume,
                )
            }
        }
    };
}
