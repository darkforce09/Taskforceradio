//! Unit tests for the audio drivers, on the in-process plugin.

use tfar_ts3_host::ClientId;

use super::*;
use crate::mock_host::in_process_plugin::start_in_process_host;

#[test]
fn post_process_voice_returns_the_samples_and_mask_the_plugin_left() {
    let host = start_in_process_host();
    let outcome = host
        .post_process_voice(ClientId(2), &[1, -2, 3, i16::MAX], 2)
        .unwrap();
    assert_eq!(
        outcome,
        VoiceBlockOutcome {
            samples: vec![2, -4, 6, i16::MAX],
            channel_fill_mask: 0b01,
        }
    );
}

#[test]
fn captured_voice_returns_the_samples_and_edited_flags() {
    let host = start_in_process_host();
    let outcome = host.captured_voice(&[5, -5, 0], 1, 2).unwrap();
    assert_eq!(
        outcome,
        CapturedVoiceOutcome {
            samples: vec![-5, 5, 0],
            edited: 3,
        }
    );
}

#[test]
fn mixed_playback_returns_the_samples_and_mask() {
    let host = start_in_process_host();
    let outcome = host.mixed_playback(&[9, 9, 9, 9], 2, 0b11).unwrap();
    assert_eq!(
        outcome,
        VoiceBlockOutcome {
            samples: vec![0, 0, 0, 0],
            channel_fill_mask: 0,
        }
    );
}

#[test]
fn custom_rolloff_starts_from_full_volume() {
    let host = start_in_process_host();
    let volume = host.custom_3d_rolloff(ClientId(2), 3.0).unwrap();
    assert!((volume - 0.25).abs() < f32::EPSILON);
}

#[test]
fn ragged_or_channelless_blocks_are_errors() {
    let host = start_in_process_host();
    assert!(matches!(
        host.post_process_voice(ClientId(2), &[1, 2, 3], 2),
        Err(Error::InvalidVoiceBlock {
            sample_count: 3,
            channels: 2
        })
    ));
    assert!(matches!(
        host.captured_voice(&[1], 0, 0),
        Err(Error::InvalidVoiceBlock { .. })
    ));
}

#[test]
fn speakers_are_one_bit_per_channel_and_the_mask_covers_them_all() {
    let stereo = block_shape(4, 2).unwrap();
    assert_eq!(stereo.frames, 2);
    assert_eq!(stereo.channels, 2);
    assert_eq!(stereo.speakers, vec![0b01, 0b10]);
    assert_eq!(stereo.all_channels_mask, 0b11);
    let wide = block_shape(32, 32).unwrap();
    assert_eq!(wide.all_channels_mask, u32::MAX);
    assert_eq!(wide.speakers.last(), Some(&(1 << 31)));
}

#[test]
fn a_missing_audio_export_is_an_error() {
    let mut exports = crate::mock_host::in_process_plugin::in_process_exports();
    exports.on_custom_3d_rolloff_calculation_client_event = None;
    let (_users, host) = crate::mock_host::in_process_plugin::try_start(&exports, 0);
    let host = host.unwrap();
    assert!(matches!(
        host.custom_3d_rolloff(ClientId(2), 1.0),
        Err(Error::MissingExport { .. })
    ));
}
