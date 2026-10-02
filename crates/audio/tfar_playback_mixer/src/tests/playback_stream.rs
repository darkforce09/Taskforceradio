//! Unit tests for `PlaybackStream` in `tfar_playback_mixer`.

use super::*;

#[test]
fn appended_frames_come_out_in_order() {
    let mut stream = PlaybackStream::new("my_radio_voice");
    stream.append(&[1, 1, 2, 2]);
    stream.append(&[3, 3]);
    assert_eq!(stream.queued_frames(), 3);
    let mut block = [0_i16; 4];
    assert!(stream.mix_front_into(&mut block, 2));
    assert_eq!(block, [1, 1, 2, 2]);
    assert_eq!(stream.queued_frames(), 1);
    let mut block = [0_i16; 4];
    assert!(stream.mix_front_into(&mut block, 2));
    assert_eq!(block, [3, 3, 0, 0]);
    assert!(
        !stream.mix_front_into(&mut block, 2),
        "an empty stream mixes nothing"
    );
}

#[test]
fn the_stream_stays_bounded_and_keeps_the_newest_frames() {
    let mut stream = PlaybackStream::new("my_radio_voice");
    let capacity = stream.samples.capacity();
    // 960 frames per block (20 ms), 100 blocks: 2 s of audio into a 500 ms stream.
    for block_number in 0..100_i16 {
        let block = vec![block_number; 960 * 2];
        stream.append(&block);
        assert!(stream.queued_frames() <= STREAM_CAPACITY_FRAMES);
    }
    assert_eq!(stream.queued_frames(), STREAM_CAPACITY_FRAMES);
    assert_eq!(
        stream.samples.capacity(),
        capacity,
        "the queue never reallocates"
    );
    // 24000 frames are 25 blocks of 960: the oldest kept frame is from block 75.
    assert_eq!(stream.samples.front(), Some(&75));
    assert_eq!(stream.samples.back(), Some(&99));
}

#[test]
fn an_append_larger_than_the_capacity_keeps_its_tail() {
    let mut stream = PlaybackStream::new("my_radio_voice");
    stream.append(&[7, 7]);
    let mut huge: Vec<i16> = vec![1; STREAM_CAPACITY_FRAMES * 2];
    huge.extend([2, 2]);
    stream.append(&huge);
    assert_eq!(stream.queued_frames(), STREAM_CAPACITY_FRAMES);
    assert_eq!(stream.samples.back(), Some(&2));
    assert!(!stream.samples.contains(&7));
}

#[test]
fn frames_stay_aligned_after_dropping() {
    let mut stream = PlaybackStream::new("my_radio_voice");
    let block: Vec<i16> = (0..1000).flat_map(|_| [10_i16, -10]).collect();
    for _ in 0..30 {
        stream.append(&block);
    }
    let mut out = [0_i16; 8];
    stream.mix_front_into(&mut out, 2);
    assert_eq!(out, [10, -10, 10, -10, 10, -10, 10, -10]);
}
