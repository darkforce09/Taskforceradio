//! Unit tests for the `i16` and `f32` conversion in `tfar_audio::sample_buffer`.

use super::*;

#[test]
fn input_divides_by_shrt_max() {
    assert_eq!(sample_to_float(i16::MAX), 1.0);
    assert_eq!(sample_to_float(0), 0.0);
    assert_eq!(sample_to_float(-32_767), -1.0);
    assert!(sample_to_float(i16::MIN) < -1.0);
}

#[test]
fn output_clamps_to_unit_range_then_multiplies_by_shrt_max_minus_one() {
    assert_eq!(float_to_sample(1.0), 32_766);
    assert_eq!(float_to_sample(-1.0), -32_766);
    assert_eq!(float_to_sample(7.5), 32_766);
    assert_eq!(float_to_sample(-7.5), -32_766);
    assert_eq!(float_to_sample(f32::INFINITY), 32_766);
    assert_eq!(float_to_sample(f32::NEG_INFINITY), -32_766);
    assert_eq!(float_to_sample(0.5), 16_383);
    assert_eq!(float_to_sample(f32::NAN), 0);
}

#[test]
fn a_round_trip_moves_every_sample_by_at_most_one_step() {
    for sample in -32_767..=i16::MAX {
        let back = float_to_sample(sample_to_float(sample));
        let error = (i32::from(back) - i32::from(sample)).abs();
        assert!(error <= 1, "{sample} came back as {back}");
    }
}

#[test]
fn the_most_negative_sample_comes_back_clamped_to_minus_32766() {
    assert_eq!(float_to_sample(sample_to_float(i16::MIN)), -32_766);
}

#[test]
fn planar_conversion_splits_left_and_right() {
    let mut stereo = StereoBuffer::with_frames(2);
    stereo
        .as_mut_slice()
        .copy_from_slice(&[i16::MAX, 0, -32_767, i16::MAX]);
    let mut left = [9.0; 3];
    let mut right = [9.0; 3];
    to_float_planar(&stereo, &mut left, &mut right).unwrap();
    assert_eq!(left, [1.0, -1.0, 9.0]);
    assert_eq!(right, [0.0, 1.0, 9.0]);
}

#[test]
fn planar_conversion_joins_left_and_right_for_the_buffer_frame_count() {
    let mut stereo = StereoBuffer::with_frames(2);
    from_float_planar(&[1.0, -2.0, 0.25], &[0.5, 0.0, 0.25], &mut stereo).unwrap();
    assert_eq!(stereo.frames(), 2);
    assert_eq!(stereo.as_slice(), &[32_766, 16_383, -32_766, 0]);
}

#[test]
fn a_planar_round_trip_moves_every_sample_by_at_most_one_step() {
    let original = [i16::MAX, -32_767, 12_345, -12_345, 1, -1, 0, 20_000];
    let mut stereo = StereoBuffer::with_frames(4);
    stereo.as_mut_slice().copy_from_slice(&original);
    let mut left = [0.0; 4];
    let mut right = [0.0; 4];
    to_float_planar(&stereo, &mut left, &mut right).unwrap();
    from_float_planar(&left, &right, &mut stereo).unwrap();
    for (back, sample) in stereo.as_slice().iter().zip(original) {
        assert!((i32::from(*back) - i32::from(sample)).abs() <= 1);
    }
}

#[test]
fn a_short_planar_slice_is_an_error_and_nothing_is_written() {
    let mut stereo = StereoBuffer::with_frames(3);
    stereo.as_mut_slice().copy_from_slice(&[1, 2, 3, 4, 5, 6]);
    let mut left = [9.0; 3];
    let mut right = [9.0; 2];
    assert_eq!(
        to_float_planar(&stereo, &mut left, &mut right),
        Err(Error::PlanarSliceTooShort {
            channel: "right",
            needed: 3,
            actual: 2,
        })
    );
    assert_eq!(left, [9.0; 3]);
    assert_eq!(
        from_float_planar(&[0.0; 2], &[0.0; 3], &mut stereo),
        Err(Error::PlanarSliceTooShort {
            channel: "left",
            needed: 3,
            actual: 2,
        })
    );
    assert_eq!(stereo.as_slice(), &[1, 2, 3, 4, 5, 6]);
}
