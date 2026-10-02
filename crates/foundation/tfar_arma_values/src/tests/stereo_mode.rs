//! Unit tests for `StereoMode`.

use super::*;

#[test]
fn wire_values_map_to_their_modes() {
    assert_eq!(StereoMode::from_wire_value(0), StereoMode::Both);
    assert_eq!(StereoMode::from_wire_value(1), StereoMode::Left);
    assert_eq!(StereoMode::from_wire_value(2), StereoMode::Right);
}

#[test]
fn any_other_wire_value_is_both() {
    for value in [-1, 3, 42, i32::MIN, i32::MAX] {
        assert_eq!(StereoMode::from_wire_value(value), StereoMode::Both);
    }
}

#[test]
fn wire_value_round_trips() {
    for mode in [StereoMode::Both, StereoMode::Left, StereoMode::Right] {
        assert_eq!(StereoMode::from_wire_value(mode.wire_value()), mode);
    }
}

#[test]
fn the_default_is_both() {
    assert_eq!(StereoMode::default(), StereoMode::Both);
}
