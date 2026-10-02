//! Unit tests for `StereoBuffer` in `tfar_sample_buffer`.

use super::*;

#[test]
fn a_new_buffer_holds_silent_frames_of_two_samples() {
    let buffer = StereoBuffer::with_frames(3);
    assert_eq!(buffer.frames(), 3);
    assert_eq!(buffer.as_slice(), &[0; 6]);
}

#[test]
fn clear_silences_every_sample_and_keeps_the_frame_count() {
    let mut buffer = StereoBuffer::with_frames(2);
    buffer.as_mut_slice().copy_from_slice(&[1, -2, 3, -4]);
    buffer.clear();
    assert_eq!(buffer.frames(), 2);
    assert_eq!(buffer.as_slice(), &[0, 0, 0, 0]);
}

#[test]
fn set_frames_keeps_the_remaining_frames_and_silences_new_ones() {
    let mut buffer = StereoBuffer::with_frames(2);
    buffer.as_mut_slice().copy_from_slice(&[1, 2, 3, 4]);
    buffer.set_frames(1);
    assert_eq!(buffer.as_slice(), &[1, 2]);
    buffer.set_frames(3);
    assert_eq!(buffer.as_slice(), &[1, 2, 0, 0, 0, 0]);
}

#[test]
fn copy_from_makes_an_exact_copy_including_the_frame_count() {
    let mut source = StereoBuffer::with_frames(2);
    source.as_mut_slice().copy_from_slice(&[5, 6, 7, 8]);
    let mut target = StereoBuffer::with_frames(5);
    target.copy_from(&source);
    assert_eq!(target, source);
    assert_eq!(target.frames(), 2);
}

#[test]
fn a_reserved_buffer_does_not_reallocate_in_steady_state() {
    let mut buffer = StereoBuffer::default();
    buffer.reserve_frames(960);
    assert!(buffer.capacity_frames() >= 960);
    let address_before = buffer.as_slice().as_ptr();
    let source = StereoBuffer::with_frames(960);
    for frames in [480, 960, 1, 960] {
        buffer.set_frames(frames);
        buffer.copy_from(&source);
        assert_eq!(buffer.as_slice().as_ptr(), address_before);
    }
}

#[test]
fn frame_pairs_split_the_samples_into_left_and_right() {
    let mut buffer = StereoBuffer::with_frames(2);
    buffer.as_mut_slice().copy_from_slice(&[1, 2, 3, 4]);
    assert_eq!(buffer.frame_pairs(), &[[1, 2], [3, 4]]);
    buffer.frame_pairs_mut()[1] = [9, 10];
    assert_eq!(buffer.as_slice(), &[1, 2, 9, 10]);
}
