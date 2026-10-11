//! Unit tests for the TeamSpeak channel adapters in `tfar_audio::sample_buffer`.

use super::*;

fn stereo_from(samples: &[i16]) -> StereoBuffer {
    let mut stereo = StereoBuffer::with_frames(samples.len() / 2);
    stereo.as_mut_slice().copy_from_slice(samples);
    stereo
}

#[test]
fn mono_input_is_duplicated_into_both_channels_without_reading_past_the_slice() {
    // Exactly `frames` samples: an adapter that reads channels 0 and 1 of a mono frame would
    // need one sample past the end.
    let mono = [100, -200, 300];
    let mut stereo = StereoBuffer::default();
    copy_to_stereo(&mono, 1, 3, &mut stereo).unwrap();
    assert_eq!(stereo.as_slice(), &[100, 100, -200, -200, 300, 300]);
}

#[test]
fn stereo_input_is_copied_frame_for_frame() {
    let samples = [1, 2, 3, 4, 5, 6];
    let mut stereo = StereoBuffer::with_frames(10);
    copy_to_stereo(&samples, 2, 3, &mut stereo).unwrap();
    assert_eq!(stereo.as_slice(), &[1, 2, 3, 4, 5, 6]);
}

#[test]
fn six_channel_input_keeps_channels_zero_and_one() {
    let samples = [
        10, 11, 12, 13, 14, 15, //
        20, 21, 22, 23, 24, 25,
    ];
    let mut stereo = StereoBuffer::default();
    copy_to_stereo(&samples, 6, 2, &mut stereo).unwrap();
    assert_eq!(stereo.as_slice(), &[10, 11, 20, 21]);
}

#[test]
fn samples_beyond_the_block_are_ignored_on_the_way_in() {
    let samples = [1, 2, 3, 4, 99, 99];
    let mut stereo = StereoBuffer::default();
    copy_to_stereo(&samples, 2, 2, &mut stereo).unwrap();
    assert_eq!(stereo.as_slice(), &[1, 2, 3, 4]);
}

#[test]
fn a_slice_shorter_than_channels_times_frames_is_an_error() {
    let samples = [1, 2, 3, 4, 5];
    let mut stereo = stereo_from(&[7, 7]);
    let error = copy_to_stereo(&samples, 2, 3, &mut stereo).unwrap_err();
    assert_eq!(
        error,
        Error::SampleSliceTooShort {
            needed: 6,
            actual: 5,
            channels: 2,
            frames: 3,
        }
    );
    assert_eq!(stereo.as_slice(), &[7, 7], "the output stays untouched");
}

#[test]
fn a_mono_slice_shorter_than_its_frame_count_is_an_error() {
    let mut stereo = StereoBuffer::default();
    let error = copy_to_stereo(&[1, 2], 1, 3, &mut stereo).unwrap_err();
    assert!(matches!(
        error,
        Error::SampleSliceTooShort {
            needed: 3,
            actual: 2,
            ..
        }
    ));
}

#[test]
fn zero_channels_is_an_error() {
    let mut stereo = StereoBuffer::default();
    assert_eq!(
        copy_to_stereo(&[1, 2], 0, 1, &mut stereo),
        Err(Error::NoChannels)
    );
    let mut samples = [1, 2];
    assert_eq!(
        write_back_from_stereo(&stereo, &mut samples, 0, 0),
        Err(Error::NoChannels)
    );
}

#[test]
fn an_overflowing_sample_count_is_an_error() {
    let mut stereo = StereoBuffer::default();
    assert_eq!(
        copy_to_stereo(&[], 2, usize::MAX, &mut stereo),
        Err(Error::SampleCountOverflow {
            channels: 2,
            frames: usize::MAX,
        })
    );
}

#[test]
fn mono_write_back_averages_left_and_right_within_the_mono_slice() {
    let stereo = stereo_from(&[100, 100, 100, -100, i16::MAX, i16::MAX, -3, 0]);
    let mut mono = [0; 4];
    write_back_from_stereo(&stereo, &mut mono, 1, 4).unwrap();
    // Equal channels come back exactly, opposite ones cancel, full scale never overflows, and the
    // half is rounded toward zero.
    assert_eq!(mono, [100, 0, i16::MAX, -1]);
}

#[test]
fn mono_write_back_never_touches_samples_past_the_block() {
    let stereo = stereo_from(&[5, 5, 6, 6]);
    let mut mono = [0, 0, 42];
    write_back_from_stereo(&stereo, &mut mono, 1, 2).unwrap();
    assert_eq!(mono, [5, 6, 42]);
}

#[test]
fn stereo_write_back_copies_frame_for_frame() {
    let stereo = stereo_from(&[1, 2, 3, 4]);
    let mut samples = [0; 4];
    write_back_from_stereo(&stereo, &mut samples, 2, 2).unwrap();
    assert_eq!(samples, [1, 2, 3, 4]);
}

#[test]
fn six_channel_write_back_fills_channels_zero_and_one_and_zeroes_the_rest() {
    let stereo = stereo_from(&[10, 11, 20, 21]);
    let mut samples = [99; 12];
    write_back_from_stereo(&stereo, &mut samples, 6, 2).unwrap();
    assert_eq!(
        samples,
        [
            10, 11, 0, 0, 0, 0, //
            20, 21, 0, 0, 0, 0,
        ]
    );
}

#[test]
fn a_short_slice_on_write_back_is_an_error_and_leaves_it_untouched() {
    let stereo = stereo_from(&[1, 2, 3, 4]);
    let mut samples = [9; 3];
    let error = write_back_from_stereo(&stereo, &mut samples, 2, 2).unwrap_err();
    assert_eq!(
        error,
        Error::SampleSliceTooShort {
            needed: 4,
            actual: 3,
            channels: 2,
            frames: 2,
        }
    );
    assert_eq!(samples, [9; 3]);
}

#[test]
fn a_stereo_buffer_shorter_than_the_block_is_an_error_on_write_back() {
    let stereo = stereo_from(&[1, 2]);
    let mut samples = [9; 4];
    assert_eq!(
        write_back_from_stereo(&stereo, &mut samples, 2, 2),
        Err(Error::StereoBufferTooShort {
            needed: 2,
            actual: 1,
        })
    );
    assert_eq!(samples, [9; 4]);
}

#[test]
fn a_mono_round_trip_through_stereo_returns_the_original_samples() {
    let original = [i16::MIN, -1, 0, 1, i16::MAX];
    let mut stereo = StereoBuffer::default();
    copy_to_stereo(&original, 1, original.len(), &mut stereo).unwrap();
    let mut mono = [0; 5];
    write_back_from_stereo(&stereo, &mut mono, 1, original.len()).unwrap();
    assert_eq!(mono, original);
}

#[test]
fn mono_to_stereo_duplicates_every_sample_and_sets_the_frame_count() {
    let mut stereo = StereoBuffer::with_frames(8);
    mono_to_stereo(&[7, -8, 9], &mut stereo);
    assert_eq!(stereo.frames(), 3);
    assert_eq!(stereo.as_slice(), &[7, 7, -8, -8, 9, 9]);
}
