//! Unit tests for `StreamProducer` and its ring: whole frames only, drain order, the bound that
//! drops the oldest frames, and a push that never waits for the reading side.

use std::time::{Duration, Instant};

use super::*;

/// Interleaved stereo frames `first..first + count`, frame `n` holding `(n, -n)`.
fn numbered_frames(first: usize, count: usize) -> Vec<i16> {
    (first..first + count)
        .flat_map(|frame| {
            let left = i16::try_from(frame).unwrap();
            [left, -left]
        })
        .collect()
}

/// Mixes one block of `frames` stereo frames from `stream` into silence and returns it.
fn drain_block(stream: &ProducerStream, frames: usize) -> Vec<i16> {
    let mut block = vec![0_i16; frames * 2];
    stream.mix_front_into(&mut block, 2);
    block
}

#[test]
fn the_producer_moves_to_the_audio_thread() {
    fn assert_send<T: Send>() {}
    assert_send::<StreamProducer>();
}

#[test]
fn pushed_frames_drain_in_order_across_blocks() {
    let (mut producer, stream) = StreamProducer::new("voice");
    assert_eq!(producer.name(), "voice");
    assert_eq!(producer.push(&numbered_frames(1, 3)), 6);
    assert_eq!(producer.push(&numbered_frames(4, 3)), 6);
    assert_eq!(stream.queued_frames(), 6);
    assert_eq!(drain_block(&stream, 4), numbered_frames(1, 4));
    assert_eq!(drain_block(&stream, 4), [5, -5, 6, -6, 0, 0, 0, 0]);
    assert_eq!(stream.queued_frames(), 0);
    assert_eq!(
        drain_block(&stream, 2),
        [0, 0, 0, 0],
        "an empty ring mixes nothing"
    );
}

#[test]
fn a_trailing_odd_sample_is_ignored() {
    let (mut producer, stream) = StreamProducer::new("voice");
    assert_eq!(producer.push(&[1, -1, 2]), 2);
    assert_eq!(producer.push(&[]), 0);
    assert_eq!(producer.push(&[7]), 0);
    assert_eq!(stream.queued_frames(), 1);
    assert_eq!(drain_block(&stream, 2), [1, -1, 0, 0]);
}

#[test]
fn beyond_the_capacity_the_oldest_frames_are_dropped() {
    let (mut producer, stream) = StreamProducer::new("voice");
    let extra = 4_800;
    let mut pushed = 0;
    for first in (0..STREAM_CAPACITY_FRAMES + extra).step_by(960) {
        let count = 960.min(STREAM_CAPACITY_FRAMES + extra - first);
        pushed += producer.push(&numbered_frames(first, count));
    }
    assert_eq!(
        pushed,
        (STREAM_CAPACITY_FRAMES + extra) * 2,
        "every frame got in"
    );
    assert_eq!(stream.queued_frames(), STREAM_CAPACITY_FRAMES);
    assert_eq!(
        drain_block(&stream, 3),
        numbered_frames(extra, 3),
        "the front is the oldest frame kept, 500 ms before the newest"
    );
}

#[test]
fn one_push_larger_than_the_capacity_keeps_its_newest_frames() {
    let (mut producer, stream) = StreamProducer::new("voice");
    let total = STREAM_CAPACITY_FRAMES + 100;
    assert_eq!(
        producer.push(&numbered_frames(0, total)),
        STREAM_CAPACITY_FRAMES * 2
    );
    assert_eq!(stream.queued_frames(), STREAM_CAPACITY_FRAMES);
    assert_eq!(drain_block(&stream, 2), numbered_frames(100, 2));
}

#[test]
fn a_push_while_the_reading_side_is_held_returns_at_once_and_keeps_what_fits() {
    let (mut producer, stream) = StreamProducer::new("voice");
    producer.push(&numbered_frames(0, STREAM_CAPACITY_FRAMES - 2));
    let held = stream.consumer.lock();
    let started = Instant::now();
    let accepted = producer.push(&numbered_frames(30_000, 5));
    assert!(started.elapsed() < Duration::from_millis(50));
    assert_eq!(
        accepted, 4,
        "two frames fit; nothing old can be dropped meanwhile"
    );
    drop(held);
    assert_eq!(stream.queued_frames(), STREAM_CAPACITY_FRAMES);
    assert_eq!(
        producer.push(&numbered_frames(1, 1)),
        2,
        "with the lock free the oldest frame makes room again"
    );
}

#[test]
fn the_stream_is_spent_once_the_producer_is_gone_and_drained() {
    let (mut producer, stream) = StreamProducer::new("voice");
    producer.push(&numbered_frames(1, 2));
    assert!(!stream.is_spent());
    drop(producer);
    assert!(!stream.is_spent(), "frames are still queued");
    assert_eq!(drain_block(&stream, 2), numbered_frames(1, 2));
    assert!(stream.is_spent());
}
