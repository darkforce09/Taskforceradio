//! Unit tests for the mixed-playback block helpers in `tfar_playback_mixer`.

use super::*;

#[test]
fn a_block_is_the_first_channels_times_frames_samples() {
    let mut samples = [1, 2, 3, 4, 5, 6, 7];
    let block = checked_block(&mut samples, 3, 2).unwrap();
    assert_eq!(block, &[1, 2, 3, 4, 5, 6]);
}

#[test]
fn bad_counts_are_errors() {
    let mut samples = [0_i16; 4];
    assert!(matches!(
        checked_block(&mut samples, 0, 1),
        Err(Error::NoChannels)
    ));
    assert!(matches!(
        checked_block(&mut samples, 2, usize::MAX),
        Err(Error::SampleCountOverflow { .. })
    ));
    assert!(matches!(
        checked_block(&mut samples, 2, 3),
        Err(Error::SampleSliceTooShort {
            needed: 6,
            actual: 4
        })
    ));
}

#[test]
fn only_the_unfilled_front_channels_are_silenced() {
    let mut block = [9_i16; 8];
    silence_unfilled_channels(&mut block, 4, 0);
    assert_eq!(block, [0, 0, 9, 9, 0, 0, 9, 9]);

    let mut block = [9_i16; 4];
    silence_unfilled_channels(&mut block, 2, 0b01);
    assert_eq!(block, [9, 0, 9, 0], "channel 0 is filled, channel 1 is not");

    let mut block = [9_i16; 4];
    silence_unfilled_channels(&mut block, 2, 0b11);
    assert_eq!(block, [9; 4]);

    let mut mono = [9_i16; 3];
    silence_unfilled_channels(&mut mono, 1, 0);
    assert_eq!(mono, [0; 3]);
}

#[test]
fn stereo_frames_are_added_into_channels_zero_and_one_only() {
    let mut block = [1_i16, 2, 50, 3, 4, 60];
    let mixed = mix_stereo_samples(&mut block, 3, [10, 20, 30, 40].into_iter());
    assert_eq!(mixed, 2);
    assert_eq!(block, [11, 22, 50, 33, 44, 60]);
}

#[test]
fn mixing_saturates_instead_of_wrapping() {
    let mut block = [i16::MAX - 1, i16::MIN + 1];
    mix_stereo_samples(&mut block, 2, [100, -100].into_iter());
    assert_eq!(block, [i16::MAX, i16::MIN]);
}

#[test]
fn a_mono_block_receives_the_average_of_left_and_right() {
    let mut block = [0_i16, 10];
    let mixed = mix_stereo_samples(&mut block, 1, [100, 300, -3, 0].into_iter());
    assert_eq!(mixed, 2);
    assert_eq!(block, [200, 9]);
}

#[test]
fn mixing_stops_at_the_end_of_the_block_or_the_source() {
    let mut block = [0_i16; 4];
    assert_eq!(mix_stereo_samples(&mut block, 2, [1, 2].into_iter()), 1);
    assert_eq!(block, [1, 2, 0, 0]);
    let mut block = [0_i16; 2];
    assert_eq!(
        mix_stereo_samples(&mut block, 2, [1, 2, 3, 4].into_iter()),
        1
    );
    // A half frame at the end of the source is never mixed.
    let mut block = [0_i16; 4];
    assert_eq!(mix_stereo_samples(&mut block, 2, [1, 2, 3].into_iter()), 1);
    assert_eq!(block, [1, 2, 0, 0]);
}

#[test]
fn the_written_bits_are_the_channels_that_exist() {
    assert_eq!(written_channel_bits(1), 0b01);
    assert_eq!(written_channel_bits(2), 0b11);
    assert_eq!(written_channel_bits(6), 0b11);
}
