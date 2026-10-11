//! Unit tests for the tangent parser, against `CommandProcessor.cpp:293-376`
//! (`reference/tfar_1_0/ts/src/`) and the messages `fnc_doSRTransmit.sqf:57-63`,
//! `fnc_doSRTransmitEnd.sqf:40-45` and `fnc_doLRTransmit.sqf` build.

use tfar_arma_values::RadioSubtype;

use crate::error::Error;
use crate::{CommandName, GameCommand, TangentKind, TangentPress, parse};

fn press(text: &str) -> TangentPress {
    match parse(text).unwrap() {
        GameCommand::Tangent(press) => press,
        other => panic!("not a tangent: {other:?}"),
    }
}

#[test]
fn a_press_with_a_class_name_reads_six_tokens() {
    let press = press("TANGENT\tPRESSED\t87.6_1\t5000.5\tdigital\ttf_anprc152_1");
    assert_eq!(press.kind, TangentKind::ShortRange);
    assert!(press.is_pressed);
    assert_eq!(press.frequency, "87.6_1");
    assert_eq!(press.range, 5001);
    assert_eq!(press.range_text, "5000.5");
    assert_eq!(press.subtype, RadioSubtype::Digital);
    assert_eq!(press.radio_classname.as_deref(), Some("tf_anprc152_1"));
}

#[test]
fn a_release_without_a_class_name_reads_five_tokens() {
    let press = press("TANGENT_LR\tRELEASED\t30_2\t20000\tdigital_lr");
    assert_eq!(press.kind, TangentKind::LongRange);
    assert!(!press.is_pressed);
    assert_eq!(press.subtype, RadioSubtype::DigitalLongRange);
    assert_eq!(press.radio_classname, None);
}

#[test]
fn only_pressed_is_a_press() {
    // `pressed = tokens[1] == "PRESSED"sv`, CommandProcessor.cpp:298.
    assert!(!press("TANGENT\tpressed\t1\t1\tdigital").is_pressed);
}

#[test]
fn the_three_tangent_names_give_their_kinds() {
    assert_eq!(
        press("TANGENT_DD\tPRESSED\t32\t70\tdd").kind,
        TangentKind::DiverDigital
    );
    assert!(!TangentKind::DiverDigital.is_long_range());
    assert!(TangentKind::LongRange.is_long_range());
    for kind in [
        TangentKind::ShortRange,
        TangentKind::LongRange,
        TangentKind::DiverDigital,
    ] {
        assert_eq!(kind.command_name().wire_name(), kind.wire_name());
    }
}

#[test]
fn the_subtype_is_matched_by_exact_name_not_by_length() {
    // The original matches by length (plugin.cpp:116-125): "abcde" would be a phone there.
    assert_eq!(
        press("TANGENT\tPRESSED\t1\t1\tabcde").subtype,
        RadioSubtype::Unknown("abcde".to_owned())
    );
    assert_eq!(
        press("TANGENT\tPRESSED\tdirectSpeechFreq\t0\tdirectSpeech").subtype,
        RadioSubtype::Unknown("directSpeech".to_owned())
    );
}

#[test]
fn a_tangent_of_four_or_seven_tokens_is_an_error() {
    assert_eq!(
        parse("TANGENT\tPRESSED\t1\t1"),
        Err(Error::WrongTokenCount {
            command: CommandName::Tangent,
            minimum: 5,
            maximum: 6,
            found: 4
        })
    );
    assert!(parse("TANGENT\tPRESSED\t1\t1\tdigital\tradio\textra").is_err());
}

#[test]
fn a_malformed_range_is_an_error() {
    assert!(matches!(
        parse("TANGENT\tPRESSED\t1\tfar\tdigital"),
        Err(Error::Malformed { field: "range", .. })
    ));
}
