//! Unit tests for `Playback` and `PlaybackId` in `tfar_audio::playback_mixer`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

use super::*;

#[test]
fn ids_are_unique_and_increasing() {
    let first = PlaybackId::next();
    let second = PlaybackId::next();
    assert!(second.value() > first.value());
    assert_ne!(first, second);
}

#[test]
fn every_playback_gets_its_own_id() {
    let first = Playback::new(&[1, 2], |_| {}).unwrap();
    let second = Playback::new(&[1, 2], |_| {}).unwrap();
    assert_ne!(first.id(), second.id());
}

#[test]
fn the_processing_runs_at_creation_and_its_state_is_freed_there() {
    /// Stands for a filter the processing closure owns; records when it is dropped.
    struct OwnedFilter(Arc<AtomicBool>);
    impl Drop for OwnedFilter {
        fn drop(&mut self) {
            self.0.store(true, AtomicOrdering::SeqCst);
        }
    }

    let dropped = Arc::new(AtomicBool::new(false));
    let filter = OwnedFilter(Arc::clone(&dropped));
    let playback = Playback::new(&[100, 200, 300, 400], move |samples| {
        let _owned = &filter;
        crate::sample_buffer::apply_gain(samples.as_mut_slice(), 0.5);
    })
    .unwrap();
    assert!(
        dropped.load(AtomicOrdering::SeqCst),
        "the closure and its filter are gone once the playback exists"
    );
    assert_eq!(playback.samples(), &[50, 100, 150, 200]);
    assert_eq!(playback.frames(), 2);
}

#[test]
fn the_processing_may_change_the_length() {
    let playback = Playback::new(&[1, 2, 3, 4], |samples| samples.set_frames(1)).unwrap();
    assert_eq!(playback.samples(), &[1, 2]);
}

#[test]
fn an_odd_sample_count_is_an_error() {
    assert!(matches!(
        Playback::new(&[1, 2, 3], |_| {}),
        Err(Error::OddStereoSampleCount { samples: 3 })
    ));
}

#[test]
fn mixing_advances_the_cursor_until_the_playback_is_finished() {
    let mut playback = Playback::new(&[1, 1, 2, 2, 3, 3], |_| {}).unwrap();
    let mut block = [0_i16; 4];
    assert!(playback.mix_next_into(&mut block, 2));
    assert_eq!(block, [1, 1, 2, 2]);
    assert_eq!(playback.remaining_frames(), 1);
    assert!(!playback.is_finished());
    let mut block = [0_i16; 4];
    assert!(playback.mix_next_into(&mut block, 2));
    assert_eq!(block, [3, 3, 0, 0]);
    assert!(playback.is_finished());
    assert!(
        !playback.mix_next_into(&mut block, 2),
        "nothing is left to mix"
    );
}

#[test]
fn an_empty_playback_is_finished_at_once() {
    let playback = Playback::from_stereo(StereoBuffer::default());
    assert!(playback.is_finished());
}
