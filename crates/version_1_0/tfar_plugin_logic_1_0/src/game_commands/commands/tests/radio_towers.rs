//! Unit tests for the radio tower parsers, against `CommandProcessor.cpp:418-438`
//! (`reference/tfar_1_0/ts/src/`) and `fnc_pluginAddRadioTower.sqf:35`.

use crate::game_commands::error::Error;
use crate::game_commands::{GameCommand, NetworkObjectId, parse};

#[test]
fn towers_are_net_id_range_and_position() {
    let GameCommand::RadioTowerAdd(towers) =
        parse("RadioTwrAdd\t2:51;30000;[100,200,50]\n2:52;15000.5;[1,2,3]").unwrap()
    else {
        panic!("not RadioTwrAdd");
    };
    assert_eq!(towers.len(), 2);
    assert_eq!(
        towers[0].net_id,
        NetworkObjectId {
            creator: 2,
            object: 51
        }
    );
    assert!((towers[0].range - 30000.0).abs() < f64::EPSILON);
    assert_eq!(towers[0].position.to_array(), [100.0, 200.0, 50.0]);
    assert!((towers[1].range - 15000.5).abs() < f64::EPSILON);
}

#[test]
fn a_tower_without_three_parts_is_skipped() {
    // `if (antennaData.size() == 3) ... else { //#TODO log? }`, CommandProcessor.cpp:422-426.
    let GameCommand::RadioTowerAdd(towers) =
        parse("RadioTwrAdd\t2:51;30000\n2:52;100;[1,2,3]\n").unwrap()
    else {
        panic!("not RadioTwrAdd");
    };
    assert_eq!(towers.len(), 1);
    assert_eq!(towers[0].net_id.object, 52);
}

#[test]
fn a_malformed_tower_range_rejects_the_message() {
    assert!(matches!(
        parse("RadioTwrAdd\t2:51;far;[1,2,3]"),
        Err(Error::Malformed {
            field: "tower range",
            ..
        })
    ));
}

#[test]
fn removals_keep_every_record() {
    let GameCommand::RadioTowerRemove(ids) = parse("RadioTwrDel\t2:51\n2:52").unwrap() else {
        panic!("not RadioTwrDel");
    };
    assert_eq!(ids.len(), 2);
    assert_eq!(ids[1].object, 52);
    let GameCommand::RadioTowerRemove(ids) = parse("RadioTwrDel\t").unwrap() else {
        panic!("not RadioTwrDel");
    };
    assert_eq!(ids, [NetworkObjectId::default()]);
    assert!(ids[0].is_null());
}

#[test]
fn a_tower_message_without_its_list_is_an_error() {
    assert!(parse("RadioTwrAdd").is_err());
    assert!(parse("RadioTwrDel").is_err());
}
