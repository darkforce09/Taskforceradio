//! Unit tests for `TANGENT`, `TANGENT_LR` and `TANGENT_DD`, against
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:1812-1873`.

use super::*;
use crate::game_commands::error::Error;

#[test]
fn a_pressed_short_range_tangent_is_parsed() {
    // fn_onSwTangentPressed.sqf: `TANGENT\tPRESSED\t%1%2\t%3\t%4`.
    assert_eq!(
        GameCommand::parse("TANGENT\tPRESSED\t87.6_bluefor\t5000\tdigital").unwrap(),
        GameCommand::Tangent {
            kind: TangentKind::ShortRange,
            pressed: true,
            frequency: "87.6_bluefor".to_owned(),
            range: 5000,
            subtype: RadioSubtype::Digital,
        }
    );
}

#[test]
fn the_command_name_selects_the_kind() {
    // plugin.cpp:1815-1816.
    for (name, kind) in [
        ("TANGENT", TangentKind::ShortRange),
        ("TANGENT_LR", TangentKind::LongRange),
        ("TANGENT_DD", TangentKind::DiverDigital),
    ] {
        assert_eq!(TangentKind::from_command_name(name), Some(kind));
        assert_eq!(kind.command_name(), name);
        let message = format!("{name}\tRELEASED\t45\t20000\tdigital_lr");
        assert!(matches!(
            GameCommand::parse(&message).unwrap(),
            GameCommand::Tangent { kind: parsed, pressed: false, .. } if parsed == kind
        ));
    }
    assert_eq!(TangentKind::from_command_name("TANGENT_XX"), None);
}

#[test]
fn anything_but_pressed_is_a_release() {
    // `pressed = (tokens[1] == "PRESSED")`, plugin.cpp:1814.
    assert!(matches!(
        GameCommand::parse("TANGENT\tpressed\t87.6\t5000\tdigital").unwrap(),
        GameCommand::Tangent { pressed: false, .. }
    ));
}

#[test]
fn the_range_is_rounded_and_a_malformed_range_is_an_error() {
    assert!(matches!(
        GameCommand::parse("TANGENT_LR\tPRESSED\t45\t1.5e+006\tdigital_lr").unwrap(),
        GameCommand::Tangent {
            range: 1_500_000,
            ..
        }
    ));
    assert!(matches!(
        GameCommand::parse("TANGENT\tPRESSED\t87.6\tfar\tdigital"),
        Err(Error::Malformed {
            command: "TANGENT",
            field: "range",
            ..
        })
    ));
}

#[test]
fn a_tangent_with_an_empty_subtype_parses_as_an_unknown_subtype() {
    // A radio with no `tf_subtype` sends an empty last field, which getline drops; the
    // original needs `tokens.size() == 5` (plugin.cpp:1812) and refuses the press.
    assert_eq!(
        GameCommand::parse("TANGENT\tPRESSED\t87.6\t5000\t").unwrap(),
        GameCommand::Tangent {
            kind: TangentKind::ShortRange,
            pressed: true,
            frequency: "87.6".to_owned(),
            range: 5000,
            subtype: RadioSubtype::Unknown(String::new()),
        }
    );
}

#[test]
fn a_tangent_needs_four_or_five_fields() {
    for (message, found) in [
        ("TANGENT_DD\tPRESSED\t32", 3),
        ("TANGENT_DD\tPRESSED\t32\t70\tdd\textra", 6),
    ] {
        assert_eq!(
            GameCommand::parse(message).unwrap_err(),
            Error::WrongFieldCount {
                command: "TANGENT_DD",
                expected: 5,
                found
            }
        );
    }
}
