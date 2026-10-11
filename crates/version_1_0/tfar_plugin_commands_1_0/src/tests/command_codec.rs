//! Unit tests for `encode` and `decode`: byte-exact texts, round trips and the 8- and 9-token
//! tangent forms.

use super::*;

fn tangent(radio_classname: Option<&str>) -> TangentBroadcast {
    TangentBroadcast {
        kind: TangentKind::ShortRange,
        is_pressed: true,
        frequency: "87.6_1".to_owned(),
        range: 5001,
        range_text: "5000.5".to_owned(),
        subtype: RadioSubtype::Digital,
        radio_classname: radio_classname.map(str::to_owned),
        can_use_sw_radio: true,
        can_use_dd_radio: false,
        nickname: "Alpha 1".to_owned(),
    }
}

#[test]
fn a_tangent_with_a_class_name_is_the_game_text_plus_sw_dd_and_nick() {
    // `command + "\t" + sw + "\t" + dd + "\t" + nick`, CommandProcessor.cpp:326.
    assert_eq!(
        encode(&PluginCommand::Tangent(tangent(Some("tf_anprc152_1")))),
        "TANGENT\tPRESSED\t87.6_1\t5000.5\tdigital\ttf_anprc152_1\t1\t0\tAlpha 1"
    );
}

#[test]
fn a_tangent_without_a_class_name_has_eight_tokens() {
    let mut release = tangent(None);
    release.is_pressed = false;
    release.kind = TangentKind::LongRange;
    release.subtype = RadioSubtype::DigitalLongRange;
    let text = encode(&PluginCommand::Tangent(release));
    assert_eq!(
        text,
        "TANGENT_LR\tRELEASED\t87.6_1\t5000.5\tdigital_lr\t1\t0\tAlpha 1"
    );
    assert_eq!(text.split('\t').count(), 8);
}

#[test]
fn an_eight_token_tangent_reads_its_flags_from_tokens_five_and_six() {
    // The original reads canUseSW from tokens[6] and canUseDD from tokens[7] in both forms
    // (plugin.cpp:935-936), so the 8-token form takes dd as sw and the nickname as dd.
    let PluginCommand::Tangent(decoded) =
        decode("TANGENT\tRELEASED\t87.6\t3000\tdigital\t1\t0\tAlpha").unwrap()
    else {
        panic!("not a tangent");
    };
    assert!(decoded.can_use_sw_radio);
    assert!(!decoded.can_use_dd_radio);
    assert_eq!(decoded.radio_classname, None);
    assert_eq!(decoded.nickname, "Alpha");
    let PluginCommand::Tangent(decoded) =
        decode("TANGENT\tRELEASED\t87.6\t3000\tdigital\t0\t1\tAlpha").unwrap()
    else {
        panic!("not a tangent");
    };
    assert!(!decoded.can_use_sw_radio);
    assert!(decoded.can_use_dd_radio);
}

#[test]
fn a_nine_token_tangent_reads_class_name_flags_and_nick() {
    // 9 tokens: classname 5, sw 6, dd 7, nick 8 (plugin.cpp:882, :935-936).
    let PluginCommand::Tangent(decoded) =
        decode("TANGENT_DD\tPRESSED\t32\t70\tdd\ttf_rf7800str\t0\t1\tBravo").unwrap()
    else {
        panic!("not a tangent");
    };
    assert_eq!(decoded.kind, TangentKind::DiverDigital);
    assert!(decoded.is_pressed);
    assert_eq!(decoded.range, 70);
    assert_eq!(decoded.subtype, RadioSubtype::DiverDigital);
    assert_eq!(decoded.radio_classname.as_deref(), Some("tf_rf7800str"));
    assert!(!decoded.can_use_sw_radio);
    assert!(decoded.can_use_dd_radio);
    assert_eq!(decoded.nickname, "Bravo");
}

#[test]
fn every_command_round_trips_byte_for_byte() {
    let texts = [
        "TANGENT\tPRESSED\t87.6_1\t5000.5\tdigital\ttf_anprc152_1\t1\t0\tAlpha 1",
        "TANGENT_LR\tRELEASED\t30_2\t20000\tdigital_lr\t1\t1\tAlpha",
        "TANGENT_DD\tPRESSED\t32\t70\tdd\t\t0\t1\tBravo",
        "TANGENT\tPRESSED\tdirectSpeechFreq\t0\tdirectSpeech\t0\t0\tCharlie",
        "RELEASE_ALL_TANGENTS\tAlpha 1",
        "VOLUME\tAlpha 1\t20\ttrue",
        "VOLUME\tAlpha 1\t-5\tfalse",
        "REQVOL\t42",
    ];
    for text in texts {
        let command = decode(text).unwrap();
        assert_eq!(encode(&command), text, "{text:?}");
    }
}

#[test]
fn volume_is_nick_decimal_volume_and_talking() {
    // `"VOLUME\t" + myNickname + "\t" + std::to_string(myVoiceVolume) + "\t" + (start ? "true" :
    // "false")`, plugin.cpp:841.
    let command = PluginCommand::Volume {
        nickname: "Alpha".to_owned(),
        voice_volume: 40,
        is_talking: false,
    };
    assert_eq!(encode(&command), "VOLUME\tAlpha\t40\tfalse");
    // isTrue on receipt, plugin.cpp:1046.
    assert_eq!(
        decode("VOLUME\tAlpha\t40\t1"),
        Ok(PluginCommand::Volume {
            nickname: "Alpha".to_owned(),
            voice_volume: 40,
            is_talking: true
        })
    );
}

#[test]
fn release_all_and_request_volume_have_two_tokens() {
    assert_eq!(
        decode("RELEASE_ALL_TANGENTS\tAlpha"),
        Ok(PluginCommand::ReleaseAllTangents {
            nickname: "Alpha".to_owned()
        })
    );
    assert_eq!(
        decode("REQVOL\t7"),
        Ok(PluginCommand::RequestVolume {
            client_id: TeamspeakClientId(7)
        })
    );
}

#[test]
fn a_command_with_the_wrong_token_count_is_unrecognised() {
    // processPluginCommand checks name and count together (plugin.cpp:1036-1067).
    for text in [
        "TANGENT\tPRESSED\t87.6\t3000\tdigital\t1\tAlpha",
        "TANGENT\tPRESSED\t87.6\t3000\tdigital\tradio\t1\t0\tAlpha\textra",
        "RELEASE_ALL_TANGENTS",
        "VOLUME\tAlpha\t20",
        "REQVOL\t1\t2",
        "HELLO\tworld",
        "",
    ] {
        assert_eq!(
            decode(text),
            Err(Error::Unrecognised {
                text: text.to_owned()
            }),
            "{text:?}"
        );
    }
    assert_eq!(
        decode("HELLO").unwrap_err().to_string(),
        "UNKNOWN PLUGIN COMMAND HELLO"
    );
}

#[test]
fn an_eight_token_message_of_another_name_is_unrecognised() {
    assert!(matches!(
        decode("TANGENT_XX\tPRESSED\t1\t1\tdigital\t1\t0\tAlpha"),
        Err(Error::Unrecognised { .. })
    ));
}

#[test]
fn malformed_numbers_are_errors() {
    assert!(matches!(
        decode("TANGENT\tPRESSED\t1\tfar\tdigital\t1\t0\tAlpha"),
        Err(Error::Malformed { field: "range", .. })
    ));
    assert!(matches!(
        decode("VOLUME\tAlpha\tloud\ttrue"),
        Err(Error::Malformed {
            field: "voice volume",
            ..
        })
    ));
    assert!(matches!(
        decode("REQVOL\t70000"),
        Err(Error::Malformed {
            field: "client id",
            ..
        })
    ));
}

#[test]
fn any_state_but_pressed_is_a_release() {
    let PluginCommand::Tangent(decoded) =
        decode("TANGENT\tpressed\t1\t1\tdigital\t1\t0\tAlpha").unwrap()
    else {
        panic!("not a tangent");
    };
    assert!(!decoded.is_pressed);
}
