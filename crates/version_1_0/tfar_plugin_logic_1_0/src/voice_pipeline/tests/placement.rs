//! Unit tests for `Placement` and the HRTF makeup gain.

use tfar_audio::sample_buffer::StereoBuffer;
use tfar_foundation::geometry::{Direction, Position};

use super::*;

fn at(x: f64, y: f64, z: f64) -> Position {
    Position::from_array([x, y, z]).unwrap()
}

const NORTH: [f64; 3] = [0.0, 1.0, 0.0];

#[test]
fn the_makeup_gain_is_four_decibels() {
    assert!((20.0 * hrtf_makeup_gain().log10() - 4.0).abs() < 1e-12);
    assert!((hrtf_makeup_gain() - 1.584_893_192_461_113).abs() < 1e-12);
}

#[test]
fn the_hrtf_level_carries_the_makeup_and_the_fallback_does_not() {
    let view = Direction::from_array(NORTH).unwrap();
    let placement = Placement::new(at(0.0, 0.0, 0.0), view, at(0.0, 10.0, 0.0), 0.5, 2.0);
    assert!((placement.hrtf_gain - 1.0).abs() < 1e-6);
    // Straight ahead the simple ILD is 0.625 on both sides.
    let (left, right) = placement.fallback_gains;
    assert!((left - 0.3125).abs() < 1e-6 && (right - 0.3125).abs() < 1e-6);
}

#[test]
fn a_source_to_the_east_of_a_listener_facing_north_is_heard_right() {
    let view = Direction::from_array(NORTH).unwrap();
    let placement = Placement::new(at(0.0, 0.0, 0.0), view, at(10.0, 0.0, 0.0), 1.0, 1.0);
    let (left, right) = placement.fallback_gains;
    assert!(right > left, "{left} {right}");
    assert!(placement.direction.right() > 0.99);
}

#[test]
fn without_an_hrtf_source_the_fallback_gains_are_applied() {
    let view = Direction::from_array(NORTH).unwrap();
    let placement = Placement::new(at(0.0, 0.0, 0.0), view, at(10.0, 0.0, 0.0), 1.0, 1.0);
    let mut buffer = StereoBuffer::with_frames(4);
    buffer.as_mut_slice().fill(10_000);
    let mut expected = buffer.clone();
    let (left, right) = placement.fallback_gains;
    tfar_audio::sample_buffer::apply_stereo_gain(&mut expected, left, right);
    let mut rendered = StereoBuffer::default();
    placement.render(None, &mut buffer, &mut rendered);
    assert_eq!(buffer, expected);
    assert!(buffer.as_slice()[1] > buffer.as_slice()[0]);
    assert_eq!(rendered.frames(), 0);
}
