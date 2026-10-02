//! Unit tests for `NetworkObjectId`, against `NetID` (`reference/tfar_1_0/ts/src/datatypes.hpp:187-212`).

use super::*;

#[test]
fn a_net_id_is_creator_colon_object() {
    assert_eq!(
        NetworkObjectId::parse("2:390"),
        NetworkObjectId {
            creator: 2,
            object: 390
        }
    );
}

#[test]
fn the_numbers_are_read_as_atoi_reads_them() {
    assert_eq!(
        NetworkObjectId::parse(" -3:12abc"),
        NetworkObjectId {
            creator: -3,
            object: 12
        }
    );
    assert_eq!(NetworkObjectId::parse("x:y"), NetworkObjectId::default());
}

#[test]
fn a_text_without_a_colon_is_the_null_id() {
    let id = NetworkObjectId::parse("2390");
    assert_eq!(id, NetworkObjectId::default());
    assert!(id.is_null());
    assert!(NetworkObjectId::parse("").is_null());
}

#[test]
fn object_zero_is_null_whatever_the_creator() {
    assert!(NetworkObjectId::parse("5:0").is_null());
    assert!(!NetworkObjectId::parse("0:5").is_null());
}

#[test]
fn a_huge_number_saturates() {
    assert_eq!(NetworkObjectId::parse("1:99999999999").object, i32::MAX);
}
