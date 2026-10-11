//! Unit tests for the `POS` parser, against `CommandProcessor.cpp:261-280`
//! (`reference/tfar_1_0/ts/src/`) and the message `fnc_preparePositionCoordinates.sqf:97-108` builds.

use crate::game_commands::error::Error;
use crate::game_commands::{CommandName, GameCommand, PositionUpdate, VehicleDescriptor, parse};

const MESSAGE: &str =
    "POS\t Alpha 1\t[1000,2000,15.5]\t[0,1,0]\ttrue\ttrue\tfalse\t0\tno\t3\t1.5\t2\tfalse\tfalse";

fn update(text: &str) -> PositionUpdate {
    match parse(text).unwrap() {
        GameCommand::Position(update) => update,
        other => panic!("not POS: {other:?}"),
    }
}

#[test]
fn a_position_message_reads_every_field_in_order() {
    let update = update(MESSAGE);
    assert_eq!(update.nickname, "_Alpha 1");
    assert_eq!(update.position.to_array(), [1000.0, 2000.0, 15.5]);
    assert_eq!(update.view_direction.to_array(), [0.0, 1.0, 0.0]);
    assert!(update.can_speak);
    assert!(update.can_use_sw_radio);
    assert!(!update.can_use_lr_radio);
    assert!(!update.can_use_dd_radio);
    assert_eq!(update.vehicle, VehicleDescriptor::not_in_vehicle());
    assert_eq!(update.terrain_interception, 3);
    assert!((update.voice_volume - 1.5).abs() < f64::EPSILON);
    assert_eq!(update.object_interception, 2);
    assert!(!update.is_spectating);
    assert!(!update.is_enemy_to_player);
}

#[test]
fn a_dd_flag_sent_as_a_number_reads_like_is_true() {
    // `_useDd` is 0 or a boolean (fnc_preparePositionCoordinates.sqf:45); isTrue accepts "1".
    let update = update(&MESSAGE.replace("\tfalse\t0\tno", "\tfalse\t1\tno"));
    assert!(update.can_use_dd_radio);
}

#[test]
fn the_view_direction_is_normalised() {
    let update = update(&MESSAGE.replace("[0,1,0]", "[0,2,0]"));
    assert_eq!(update.view_direction.to_array(), [0.0, 1.0, 0.0]);
}

#[test]
fn a_vehicle_descriptor_is_read() {
    let update = update(&MESSAGE.replace("\tno\t", "\t2:390\x100.6\x101\x10[1,0,0]\t"));
    assert_eq!(update.vehicle.name, "2:390");
    assert_eq!(update.vehicle.intercom_slot, 1);
    assert_eq!(update.vehicle.velocity, [1.0, 0.0, 0.0]);
}

#[test]
fn a_short_pos_message_is_an_error_not_an_out_of_bounds_read() {
    // The original indexes tokens[1..13] unchecked (CommandProcessor.cpp:263-277).
    let short = "POS\tnick\t[1,2,3]";
    assert_eq!(
        parse(short),
        Err(Error::WrongTokenCount {
            command: CommandName::Position,
            minimum: 14,
            maximum: 14,
            found: 3
        })
    );
}

#[test]
fn a_malformed_pos_field_rejects_the_message() {
    assert_eq!(
        parse(&MESSAGE.replace("[1000,2000,15.5]", "[1000,north,15.5]")),
        Err(Error::Malformed {
            command: CommandName::Position,
            field: "position",
            value: "[1000,north,15.5]".to_owned()
        })
    );
    assert!(matches!(
        parse(&MESSAGE.replace("\t1.5\t", "\tloud\t")),
        Err(Error::Malformed {
            field: "voice volume",
            ..
        })
    ));
    assert!(matches!(
        parse(&MESSAGE.replace("\tno\t", "\tbroken\t")),
        Err(Error::Malformed {
            field: "vehicle",
            ..
        })
    ));
}

#[test]
fn a_rejected_sync_pos_answers_not_speaking() {
    let error = parse("POS\tnick").unwrap_err();
    assert_eq!(error.sync_reply(), "00");
}
