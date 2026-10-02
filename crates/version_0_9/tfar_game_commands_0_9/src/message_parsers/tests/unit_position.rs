//! Unit tests for `POS`, against `reference/tfar_0_9_12/ts/src/plugin.cpp:1803-1811` and the
//! message `fn_preparePositionCoordinates.sqf` builds.

use super::*;

const POSITION_MESSAGE: &str =
    "POS\tAlpha 1-1\t12.5\t-3\t1.5e+001\t90\ttrue\ttrue\tfalse\tfalse\t2:345_0.5\t3.7\t1.2\t180";

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
}

#[test]
fn a_position_message_reads_all_thirteen_fields_in_order() {
    let GameCommand::Position(update) = GameCommand::parse(POSITION_MESSAGE).unwrap() else {
        panic!("not a position update");
    };
    assert_eq!(update.nickname, "Alpha 1-1");
    assert_eq!(update.position.to_array(), [12.5, -3.0, 15.0]);
    assert_close(update.view_angle.to_degrees(), 90.0);
    assert!(update.can_speak);
    assert!(update.usable_radios.short_range);
    assert!(!update.usable_radios.long_range);
    assert!(!update.usable_radios.diver);
    assert_eq!(
        update.vehicle,
        VehicleDescriptor {
            vehicle_name: "2:345".to_owned(),
            isolation: 0.5
        }
    );
    // `std::stoi("3.7")` is 3, plugin.cpp:1810.
    assert_eq!(update.terrain_interception, 3);
    assert_close(update.voice_volume_multiplier, 1.2);
    assert_close(update.current_direction.to_degrees(), 180.0);
}

#[test]
fn a_boolean_other_than_true_is_false() {
    let message = POSITION_MESSAGE.replace("\ttrue\ttrue\t", "\tTRUE\t1\t");
    let GameCommand::Position(update) = GameCommand::parse(&message).unwrap() else {
        panic!("not a position update");
    };
    assert!(!update.can_speak);
    assert!(!update.usable_radios.short_range);
}

#[test]
fn a_malformed_coordinate_is_an_error_where_stof_would_throw() {
    // F-003: plugin.cpp:1806 `std::stof` throws on a malformed number and takes TeamSpeak down.
    let message = POSITION_MESSAGE.replace("\t12.5\t", "\tnorth\t");
    assert!(matches!(
        GameCommand::parse(&message),
        Err(Error::Malformed {
            command: "POS",
            field: "x",
            ..
        })
    ));
}

#[test]
fn every_numeric_field_is_checked() {
    for (from, to, field) in [
        ("\t90\t", "\tleft\t", "view_angle"),
        ("\t3.7\t", "\tsome\t", "terrain_interception"),
        ("\t1.2\t", "\tloud\t", "voice_volume_multiplier"),
        ("\t180", "\tback", "current_direction"),
        ("\t2:345_0.5\t", "\t2:345_x\t", "vehicle_id"),
    ] {
        let message = POSITION_MESSAGE.replace(from, to);
        match GameCommand::parse(&message) {
            Err(Error::Malformed {
                command: "POS",
                field: found,
                ..
            }) => assert_eq!(found, field),
            other => panic!("{field}: {other:?}"),
        }
    }
}

#[test]
fn a_position_message_needs_exactly_fourteen_fields() {
    let short = POSITION_MESSAGE.trim_end_matches("\t180");
    assert!(matches!(
        GameCommand::parse(short),
        Err(Error::WrongFieldCount {
            command: "POS",
            expected: 14,
            found: 13
        })
    ));
    let long = format!("{POSITION_MESSAGE}\textra");
    assert!(matches!(
        GameCommand::parse(&long),
        Err(Error::WrongFieldCount { found: 15, .. })
    ));
}
