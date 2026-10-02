//! Unit tests for `PlaybackMixer` in `tfar_playback_mixer`.

use super::*;
use crate::playback_stream::STREAM_CAPACITY_FRAMES;

fn playback_of(samples: &[i16]) -> Playback {
    Playback::new(samples, |_| {}).unwrap()
}

/// Mixes one block of `frames` stereo frames into silence and returns it.
fn mix_stereo_block(mixer: &PlaybackMixer, frames: usize) -> Vec<i16> {
    let mut block = vec![0_i16; frames * 2];
    let mut mask = 0;
    mixer.mix_into(&mut block, 2, frames, &mut mask).unwrap();
    block
}

#[test]
fn the_mixer_is_shared_between_threads() {
    fn assert_send_and_sync<T: Send + Sync>() {}
    assert_send_and_sync::<PlaybackMixer>();
    assert_send_and_sync::<crate::WaveLibrary>();
    assert_send_and_sync::<Playback>();
}

#[test]
fn two_playbacks_are_added_together_with_saturation() {
    let mixer = PlaybackMixer::new();
    mixer
        .add(playback_of(&[30_000, -30_000, 100, 200]))
        .unwrap();
    mixer.add(playback_of(&[10_000, -10_000, 1, 2])).unwrap();
    let block = mix_stereo_block(&mixer, 2);
    assert_eq!(block, [i16::MAX, i16::MIN, 101, 202]);
}

#[test]
fn an_empty_fill_mask_silences_the_front_channels_first() {
    let mixer = PlaybackMixer::new();
    mixer.add(playback_of(&[10, 20])).unwrap();
    let mut block = [999_i16, 999];
    let mut mask = 0;
    mixer.mix_into(&mut block, 2, 1, &mut mask).unwrap();
    assert_eq!(block, [10, 20], "the stale 999s are cleared before mixing");
    assert_eq!(mask, 0b11);
}

#[test]
fn a_filled_mask_keeps_teamspeaks_own_mix() {
    let mixer = PlaybackMixer::new();
    mixer.add(playback_of(&[10, 20])).unwrap();
    let mut block = [1000_i16, -1000];
    let mut mask = 0b11;
    mixer.mix_into(&mut block, 2, 1, &mut mask).unwrap();
    assert_eq!(block, [1010, -980]);
    assert_eq!(mask, 0b11);
}

#[test]
fn with_nothing_to_mix_the_mask_stays_and_unfilled_channels_are_silence() {
    let mixer = PlaybackMixer::new();
    let mut block = [5_i16, 5, 5, 5];
    let mut mask = 0;
    mixer.mix_into(&mut block, 2, 2, &mut mask).unwrap();
    assert_eq!(block, [0; 4]);
    assert_eq!(mask, 0);
    let mut block = [5_i16, 5];
    let mut mask = 0b11;
    mixer.mix_into(&mut block, 2, 1, &mut mask).unwrap();
    assert_eq!(block, [5, 5]);
}

#[test]
fn other_mask_bits_are_kept() {
    let mixer = PlaybackMixer::new();
    mixer.add(playback_of(&[1, 1])).unwrap();
    let mut block = [0_i16; 6];
    let mut mask = 0b10_0000;
    mixer.mix_into(&mut block, 6, 1, &mut mask).unwrap();
    assert_eq!(mask, 0b10_0011);
}

#[test]
fn a_mono_output_gets_the_average_and_only_its_one_bit() {
    let mixer = PlaybackMixer::new();
    mixer
        .add(playback_of(&[100, 300, -50, -150, 7, 7]))
        .unwrap();
    let mut block = [42_i16; 2];
    let mut mask = 0;
    mixer.mix_into(&mut block, 1, 2, &mut mask).unwrap();
    assert_eq!(block, [200, -100]);
    assert_eq!(mask, 0b01);
    let mut block = [0_i16; 2];
    mixer.mix_into(&mut block, 1, 2, &mut mask).unwrap();
    assert_eq!(block, [7, 0]);
}

#[test]
fn a_six_channel_output_gets_channels_zero_and_one_and_keeps_the_rest() {
    let mixer = PlaybackMixer::new();
    mixer.add(playback_of(&[1, 2, 3, 4])).unwrap();
    let mut block: Vec<i16> = vec![
        9, 9, 50, 51, 52, 53, //
        9, 9, 60, 61, 62, 63,
    ];
    let mut mask = 0;
    mixer.mix_into(&mut block, 6, 2, &mut mask).unwrap();
    assert_eq!(
        block,
        [
            1, 2, 50, 51, 52, 53, //
            3, 4, 60, 61, 62, 63,
        ]
    );
    assert_eq!(mask, 0b11);
}

#[test]
fn a_finished_playback_is_removed_and_freed_off_the_audio_thread() {
    let mixer = PlaybackMixer::new();
    mixer.add(playback_of(&[1, 1, 2, 2, 3, 3])).unwrap();
    mixer
        .add(playback_of(&[5, 5, 6, 6, 7, 7, 8, 8, 9, 9]))
        .unwrap();
    let block = mix_stereo_block(&mixer, 2);
    assert_eq!(block, [6, 6, 8, 8]);
    assert_eq!(mixer.active_playback_count(), 2);
    let block = mix_stereo_block(&mixer, 2);
    assert_eq!(block, [10, 10, 8, 8]);
    assert_eq!(
        mixer.active_playback_count(),
        1,
        "the 3-frame playback has ended"
    );
    let block = mix_stereo_block(&mixer, 2);
    assert_eq!(block, [9, 9, 0, 0]);
    assert_eq!(mixer.active_playback_count(), 0);
    assert_eq!(
        mixer.release_finished(),
        2,
        "both finished playbacks were handed back"
    );
    assert_eq!(mixer.release_finished(), 0);
    let mut block = [0_i16; 4];
    let mut mask = 0;
    mixer.mix_into(&mut block, 2, 2, &mut mask).unwrap();
    assert_eq!(mask, 0, "nothing is left to mix");
}

#[test]
fn add_frees_the_playbacks_that_have_finished() {
    let mixer = PlaybackMixer::new();
    mixer.add(playback_of(&[1, 1])).unwrap();
    mix_stereo_block(&mixer, 1);
    mixer.add(playback_of(&[2, 2])).unwrap();
    assert_eq!(
        mixer.release_finished(),
        0,
        "add has dropped the finished playback already"
    );
}

#[test]
fn a_full_queue_refuses_the_playback() {
    let mixer = PlaybackMixer::new();
    for _ in 0..PLAYBACK_QUEUE_CAPACITY {
        mixer.add(playback_of(&[1, 1])).unwrap();
    }
    let refused = playback_of(&[1, 1]);
    let refused_id = refused.id();
    assert!(matches!(
        mixer.add(refused),
        Err(Error::PlaybackQueueFull { id }) if id == refused_id
    ));
}

#[test]
fn queued_playbacks_beyond_the_active_limit_wait_their_turn() {
    let mixer = PlaybackMixer::new();
    let long: Vec<i16> = vec![1; 20];
    for _ in 0..PLAYBACK_QUEUE_CAPACITY {
        mixer.add(playback_of(&long)).unwrap();
    }
    mix_stereo_block(&mixer, 1);
    for _ in 0..PLAYBACK_QUEUE_CAPACITY {
        mixer.add(playback_of(&long)).unwrap();
    }
    let capacity = mixer.mixing.lock().active.capacity();
    let block = mix_stereo_block(&mixer, 1);
    assert_eq!(mixer.active_playback_count(), MAX_ACTIVE_PLAYBACKS);
    assert_eq!(block, [64, 64]);
    assert_eq!(
        mixer.mixing.lock().active.capacity(),
        capacity,
        "the active list never reallocates"
    );
}

#[test]
fn a_held_lock_makes_mix_into_pass_the_block_through_without_waiting() {
    let mixer = PlaybackMixer::new();
    mixer.add(playback_of(&[10, 10])).unwrap();
    mixer.append_stream("my_radio_voice", &[20, 20]).unwrap();
    let mixing = mixer.mixing.lock();
    let streams = mixer.streams.lock();
    let mut block = [1_i16, 1];
    let mut mask = 0b11;
    mixer.mix_into(&mut block, 2, 1, &mut mask).unwrap();
    assert_eq!(block, [1, 1]);
    drop(mixing);
    drop(streams);
    let mut block = [1_i16, 1];
    mixer.mix_into(&mut block, 2, 1, &mut mask).unwrap();
    assert_eq!(
        block,
        [31, 31],
        "the playback and the stream waited for this block"
    );
}

#[test]
fn bad_block_counts_are_errors_and_touch_nothing() {
    let mixer = PlaybackMixer::new();
    mixer.add(playback_of(&[10, 10])).unwrap();
    let mut block = [7_i16; 3];
    let mut mask = 0;
    assert!(matches!(
        mixer.mix_into(&mut block, 2, 2, &mut mask),
        Err(Error::SampleSliceTooShort {
            needed: 4,
            actual: 3
        })
    ));
    assert!(matches!(
        mixer.mix_into(&mut block, 0, 1, &mut mask),
        Err(Error::NoChannels)
    ));
    assert_eq!(block, [7; 3]);
    assert_eq!(mask, 0);
}

#[test]
fn a_stream_is_mixed_continuously_and_stays_bounded() {
    let mixer = PlaybackMixer::new();
    for _ in 0..100 {
        mixer
            .append_stream("my_radio_voice", &[3; 960 * 2])
            .unwrap();
    }
    assert_eq!(
        mixer.stream_queued_frames("my_radio_voice"),
        Some(STREAM_CAPACITY_FRAMES)
    );
    let block = mix_stereo_block(&mixer, 960);
    assert!(block.iter().all(|sample| *sample == 3));
    assert_eq!(
        mixer.stream_queued_frames("my_radio_voice"),
        Some(STREAM_CAPACITY_FRAMES - 960)
    );
    assert_eq!(mixer.stream_queued_frames("other"), None);
}

#[test]
fn a_stream_and_a_playback_mix_together() {
    let mixer = PlaybackMixer::new();
    mixer.add(playback_of(&[100, 200])).unwrap();
    mixer
        .append_stream("my_radio_voice", &[1, 2, 3, 4])
        .unwrap();
    let block = mix_stereo_block(&mixer, 2);
    assert_eq!(block, [101, 202, 3, 4]);
}

#[test]
fn stream_errors_are_reported() {
    let mixer = PlaybackMixer::new();
    assert!(matches!(
        mixer.append_stream("my_radio_voice", &[1, 2, 3]),
        Err(Error::OddStereoSampleCount { samples: 3 })
    ));
    for index in 0..MAX_STREAMS {
        mixer
            .append_stream(&format!("stream_{index}"), &[1, 1])
            .unwrap();
    }
    assert!(matches!(
        mixer.append_stream("one_too_many", &[1, 1]),
        Err(Error::TooManyStreams {
            maximum: MAX_STREAMS,
            ..
        })
    ));
    mixer.append_stream("stream_0", &[1, 1]).unwrap();
}
