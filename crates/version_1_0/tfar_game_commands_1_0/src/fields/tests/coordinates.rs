//! Unit tests for the coordinate readers, against `Vector3D(std::string_view)`
//! (`reference/tfar_1_0/ts/src/datatypes.cpp:31-51`) and `Vector3D::isNull` (`:116-120`).

use super::*;

#[test]
fn three_elements_are_x_y_z() {
    assert_eq!(
        parse_coordinates("[1234.5,6789,12.25]"),
        Some([1234.5, 6789.0, 12.25])
    );
}

#[test]
fn two_elements_leave_z_at_zero() {
    // case 2, datatypes.cpp:39-42.
    assert_eq!(parse_coordinates("[1,2]"), Some([1.0, 2.0, 0.0]));
}

#[test]
fn a_fourth_element_is_ignored() {
    // case 4, datatypes.cpp:44: older TF_fnc_position overrides return four elements.
    assert_eq!(parse_coordinates("[1,2,3,4]"), Some([1.0, 2.0, 3.0]));
}

#[test]
fn the_brackets_are_optional_and_spaces_allowed() {
    assert_eq!(parse_coordinates("1, 2, 3"), Some([1.0, 2.0, 3.0]));
    assert_eq!(
        parse_coordinates("[1.5e+006, -2, 0]"),
        Some([1.5e6, -2.0, 0.0])
    );
}

#[test]
fn other_shapes_are_rejected_instead_of_read_as_zero() {
    for text in ["[]", "[1]", "[1,2,3,4,5]", "[1,x,3]", "[1,2,3", "", "abc"] {
        assert_eq!(parse_coordinates(text), None, "{text:?}");
    }
}

#[test]
fn a_speaker_position_of_empty_brackets_is_on_its_owner() {
    assert_eq!(parse_speaker_position("[]"), Ok(None));
}

#[test]
fn a_speaker_position_of_zero_is_on_its_owner_as_is_null_reads_it() {
    assert_eq!(parse_speaker_position("[0,0,0]"), Ok(None));
}

#[test]
fn a_speaker_position_is_read_otherwise() {
    let position = parse_speaker_position("[10,20,3]").unwrap().unwrap();
    assert_eq!(position.to_array(), [10.0, 20.0, 3.0]);
    assert_eq!(parse_speaker_position("[10,x,3]"), Err(()));
}
