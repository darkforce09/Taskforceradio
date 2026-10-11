//! Unit tests for `PluginCommand`, against `reference/tfar_0_9_12/ts/src/plugin.cpp`: the tangent
//! broadcast (1837), the forwarded `RELEASE_ALL_TANGENTS` (1785-1788), the `VOLUME` text (2906)
//! and `processPluginCommand` (3071-3110).

use super::*;

fn tangent(kind: TangentKind, pressed: bool) -> PluginCommand {
    PluginCommand::Tangent {
        kind,
        pressed,
        frequency: "87.6_bluefor".to_owned(),
        range: 5000,
        subtype: RadioSubtype::Digital,
        nickname: "Alpha 1-1".to_owned(),
    }
}

#[test]
fn a_tangent_encodes_as_the_game_command_plus_the_nickname() {
    // `command + "\t" + myNickname`, plugin.cpp:1837, with the game command of
    // fn_onSwTangentPressed.sqf: `TANGENT\tPRESSED\t%1%2\t%3\t%4`.
    assert_eq!(
        tangent(TangentKind::ShortRange, true).encode(),
        "TANGENT\tPRESSED\t87.6_bluefor\t5000\tdigital\tAlpha 1-1"
    );
    assert_eq!(
        tangent(TangentKind::LongRange, false).encode(),
        "TANGENT_LR\tRELEASED\t87.6_bluefor\t5000\tdigital\tAlpha 1-1"
    );
    // fn_onDDTangentPressed.sqf: `TANGENT_DD\tPRESSED\t%1\t0\tdd`.
    let diver = PluginCommand::Tangent {
        kind: TangentKind::DiverDigital,
        pressed: true,
        frequency: "32".to_owned(),
        range: 0,
        subtype: RadioSubtype::DiverDigital,
        nickname: "Diver".to_owned(),
    };
    assert_eq!(diver.encode(), "TANGENT_DD\tPRESSED\t32\t0\tdd\tDiver");
}

#[test]
fn release_all_tangents_encodes_as_the_forwarded_game_command() {
    // plugin.cpp:1787 forwards fn_radioReplaceProcess.sqf's `RELEASE_ALL_TANGENTS\t%1` as is.
    let command = PluginCommand::ReleaseAllTangents {
        nickname: "Alpha".to_owned(),
    };
    assert_eq!(command.encode(), "RELEASE_ALL_TANGENTS\tAlpha");
}

#[test]
fn volume_encodes_like_the_original() {
    // "VOLUME\t" + myNickname + "\t" + std::to_string(myVoiceVolume) + "\t" + "true"/"false",
    // plugin.cpp:2906.
    let talking = PluginCommand::Volume {
        nickname: "Alpha".to_owned(),
        voice_volume: 20,
        is_talking: true,
    };
    assert_eq!(talking.encode(), "VOLUME\tAlpha\t20\ttrue");
    let silent = PluginCommand::Volume {
        nickname: "Alpha".to_owned(),
        voice_volume: -1,
        is_talking: false,
    };
    assert_eq!(silent.encode(), "VOLUME\tAlpha\t-1\tfalse");
}

#[test]
fn every_command_round_trips() {
    let commands = [
        tangent(TangentKind::ShortRange, true),
        tangent(TangentKind::LongRange, false),
        tangent(TangentKind::DiverDigital, true),
        PluginCommand::Tangent {
            kind: TangentKind::ShortRange,
            pressed: false,
            frequency: "60".to_owned(),
            range: 3000,
            subtype: RadioSubtype::Unknown("custom".to_owned()),
            nickname: "B".to_owned(),
        },
        PluginCommand::ReleaseAllTangents {
            nickname: "Alpha 1-1".to_owned(),
        },
        PluginCommand::Volume {
            nickname: "Alpha".to_owned(),
            voice_volume: 40,
            is_talking: true,
        },
        PluginCommand::Volume {
            nickname: "Alpha".to_owned(),
            voice_volume: 5,
            is_talking: false,
        },
    ];
    for command in commands {
        assert_eq!(PluginCommand::decode(&command.encode()).unwrap(), command);
    }
}

#[test]
fn a_windows_tangent_decodes_with_the_range_rounded() {
    // `parseArmaNumberToInt(tokens[3])`, plugin.cpp:2963.
    let decoded = PluginCommand::decode("TANGENT_LR\tPRESSED\t45\t20000.6\tdigital_lr\tB").unwrap();
    assert_eq!(
        decoded,
        PluginCommand::Tangent {
            kind: TangentKind::LongRange,
            pressed: true,
            frequency: "45".to_owned(),
            range: 20001,
            subtype: RadioSubtype::DigitalLongRange,
            nickname: "B".to_owned(),
        }
    );
}

#[test]
fn decoding_compares_pressed_and_true_exactly() {
    // `tokens[1] == "PRESSED"` (plugin.cpp:2958) and `isTrue` (plugin.cpp:3089).
    assert!(matches!(
        PluginCommand::decode("TANGENT\tpressed\t1\t1\tdigital\tA").unwrap(),
        PluginCommand::Tangent { pressed: false, .. }
    ));
    assert!(matches!(
        PluginCommand::decode("VOLUME\tA\t20\tTrue").unwrap(),
        PluginCommand::Volume {
            is_talking: false,
            ..
        }
    ));
}

#[test]
fn the_voice_volume_is_truncated_like_stoi() {
    // `std::stoi(volume)`, plugin.cpp:3093.
    assert!(matches!(
        PluginCommand::decode("VOLUME\tA\t20.9\ttrue").unwrap(),
        PluginCommand::Volume {
            voice_volume: 20,
            ..
        }
    ));
}

#[test]
fn a_malformed_number_is_an_error_where_stoi_would_throw() {
    assert!(matches!(
        PluginCommand::decode("VOLUME\tA\tloud\ttrue"),
        Err(Error::Malformed {
            command: "VOLUME",
            field: "voice_volume",
            ..
        })
    ));
    assert!(matches!(
        PluginCommand::decode("TANGENT_DD\tPRESSED\t32\tfar\tdd\tA"),
        Err(Error::Malformed {
            command: "TANGENT_DD",
            field: "range",
            ..
        })
    ));
}

#[test]
fn an_unknown_name_or_count_is_an_unknown_command() {
    // plugin.cpp:3076, 3080 and 3084 need 6, 2 and 4 fields; anything else is logged at 3108.
    for (text, found) in [
        ("TANGENT\tPRESSED\t1\t1\tdigital", 5),
        ("RELEASE_ALL_TANGENTS\tA\tB", 3),
        ("VOLUME\tA\t20", 3),
        ("HELLO\tA", 2),
        ("TANGENT_XX\tPRESSED\t1\t1\tdigital\tA", 6),
        ("", 0),
    ] {
        assert_eq!(
            PluginCommand::decode(text).unwrap_err(),
            Error::UnknownCommand {
                command: text.to_owned(),
                found
            },
            "{text:?}"
        );
    }
}

#[test]
fn a_trailing_empty_field_is_dropped_before_the_count_like_getline() {
    assert!(matches!(
        PluginCommand::decode("VOLUME\tA\t20\ttrue\t").unwrap(),
        PluginCommand::Volume {
            is_talking: true,
            ..
        }
    ));
}
