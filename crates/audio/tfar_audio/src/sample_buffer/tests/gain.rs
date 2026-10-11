//! Unit tests for the saturating gain in `tfar_audio::sample_buffer`.

use super::*;

#[test]
fn gain_saturates_at_the_positive_limit_instead_of_wrapping() {
    let mut samples = [20_000, 32_767, 16_384];
    apply_gain(&mut samples, 2.0);
    // A wrapping cast would turn 40000 into -25536.
    assert_eq!(samples, [i16::MAX, i16::MAX, i16::MAX]);
}

#[test]
fn gain_saturates_at_the_negative_limit_instead_of_wrapping() {
    let mut samples = [-20_000, -32_768, -16_385];
    apply_gain(&mut samples, 2.0);
    // A wrapping cast would turn -40000 into 25536.
    assert_eq!(samples, [i16::MIN, i16::MIN, i16::MIN]);
}

#[test]
fn a_negative_gain_saturates_at_both_limits() {
    let mut samples = [i16::MIN, i16::MAX];
    apply_gain(&mut samples, -1.0);
    assert_eq!(samples, [i16::MAX, -32_767]);
}

#[test]
fn gain_within_range_truncates_toward_zero_like_the_original_cast() {
    let mut samples = [3, -3, 1000, -1000];
    apply_gain(&mut samples, 0.5);
    assert_eq!(samples, [1, -1, 500, -500]);
}

#[test]
fn gain_zero_silences_the_block() {
    let mut samples = [i16::MIN, -1, 1, i16::MAX];
    apply_gain(&mut samples, 0.0);
    assert_eq!(samples, [0; 4]);
}

#[test]
fn gain_one_leaves_every_sample_untouched() {
    let original = [i16::MIN, -12_345, 0, 1, i16::MAX];
    let mut samples = original;
    apply_gain(&mut samples, 1.0);
    assert_eq!(samples, original);
}

#[test]
fn a_nan_gain_silences_the_block() {
    let mut samples = [100, -100];
    apply_gain(&mut samples, f32::NAN);
    assert_eq!(samples, [0, 0]);
}

#[test]
fn stereo_gain_scales_each_channel_by_its_own_gain_and_saturates() {
    let mut stereo = StereoBuffer::with_frames(2);
    stereo
        .as_mut_slice()
        .copy_from_slice(&[1000, 1000, 30_000, -30_000]);
    apply_stereo_gain(&mut stereo, 0.5, 2.0);
    assert_eq!(stereo.as_slice(), &[500, 2000, 15_000, i16::MIN]);
}

#[test]
fn stereo_gain_of_one_on_both_sides_leaves_the_buffer_untouched() {
    let mut stereo = StereoBuffer::with_frames(2);
    stereo
        .as_mut_slice()
        .copy_from_slice(&[i16::MIN, -7, 7, i16::MAX]);
    let original = stereo.clone();
    apply_stereo_gain(&mut stereo, 1.0, 1.0);
    assert_eq!(stereo, original);
}

#[test]
fn stereo_gain_zero_on_one_side_silences_only_that_side() {
    let mut stereo = StereoBuffer::with_frames(2);
    stereo.as_mut_slice().copy_from_slice(&[11, 22, 33, 44]);
    apply_stereo_gain(&mut stereo, 0.0, 1.0);
    assert_eq!(stereo.as_slice(), &[0, 22, 0, 44]);
}
