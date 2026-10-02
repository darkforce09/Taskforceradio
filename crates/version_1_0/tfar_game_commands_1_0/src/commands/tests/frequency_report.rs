//! Unit tests for the `FREQ` parser, against `gameData::setFreqInfos`
//! (`reference/tfar_1_0/ts/src/task_force_radio.hpp:47-69`) and `fnc_sendFrequencyInfo.sqf:126-131`.

use tfar_arma_values::StereoMode;

use super::NICKNAMES_NEVER_ADOPTED;
use crate::error::Error;
use crate::{CommandName, FrequencyReport, GameCommand, parse};

const MESSAGE: &str = "FREQ\t[\"87.6|7|0|tf_anprc152\",\"45.1|5|1\"]\t[\"30|7|2|tf_rt1523g\"]\ttrue\t20\tAlpha 1\t0.25\t7\t1\t1\t20";

fn report(text: &str) -> FrequencyReport {
    match parse(text).unwrap() {
        GameCommand::Frequencies(report) => report,
        other => panic!("not FREQ: {other:?}"),
    }
}

#[test]
fn a_frequency_message_reads_every_field_at_the_original_index() {
    let report = report(MESSAGE);
    assert_eq!(report.sw_frequencies.len(), 2);
    let first = &report.sw_frequencies[0];
    assert_eq!(first.frequency, "87.6");
    assert!((first.volume - 7.0).abs() < f64::EPSILON);
    assert_eq!(first.stereo_mode, StereoMode::Both);
    assert_eq!(first.radio_classname.as_deref(), Some("tf_anprc152"));
    assert_eq!(report.sw_frequencies[1].radio_classname, None);
    assert_eq!(report.lr_frequencies.len(), 1);
    assert_eq!(report.lr_frequencies[0].stereo_mode, StereoMode::Right);
    assert!(report.is_alive);
    assert_eq!(report.voice_volume, 20);
    assert_eq!(report.nickname, "Alpha 1");
    assert!((report.waves_level - 0.25).abs() < f64::EPSILON);
    assert!((report.terrain_interception_coefficient - 7.0).abs() < f64::EPSILON);
    assert!((report.global_volume - 1.0).abs() < f64::EPSILON);
    assert!((report.receiving_distance_multiplier - 1.0).abs() < f64::EPSILON);
    assert!((report.speaker_distance - 20.0).abs() < f64::EPSILON);
}

#[test]
fn alive_is_true_only_for_exactly_true() {
    // `alive = tokens[3] == "true"`, task_force_radio.hpp:55; "1" is not alive here.
    assert!(!report(&MESSAGE.replace("\ttrue\t20", "\t1\t20")).is_alive);
    assert!(!report(&MESSAGE.replace("\ttrue\t20", "\tfalse\t20")).is_alive);
}

#[test]
fn empty_radio_lists_are_empty() {
    let report = report(&MESSAGE.replace(
        "[\"87.6|7|0|tf_anprc152\",\"45.1|5|1\"]\t[\"30|7|2|tf_rt1523g\"]",
        "[]\t[]",
    ));
    assert_eq!(report.sw_frequencies.len(), 0);
    assert_eq!(report.lr_frequencies.len(), 0);
}

#[test]
fn a_short_freq_message_is_an_error() {
    // The original indexes tokens[1..10] unchecked (task_force_radio.hpp:53-68).
    assert_eq!(
        parse("FREQ\t[]\t[]\ttrue"),
        Err(Error::WrongTokenCount {
            command: CommandName::Frequencies,
            minimum: 11,
            maximum: 11,
            found: 4
        })
    );
}

#[test]
fn a_malformed_freq_field_rejects_the_message() {
    assert!(matches!(
        parse(&MESSAGE.replace("\t0.25\t", "\tcalm\t")),
        Err(Error::Malformed {
            field: "waves level",
            ..
        })
    ));
    assert!(matches!(
        parse(&MESSAGE.replace("[\"30|7|2|tf_rt1523g\"]", "30|7|2")),
        Err(Error::Malformed {
            field: "long-range frequencies",
            ..
        })
    ));
}

#[test]
fn the_error_nicknames_are_never_adopted() {
    // CommandProcessor.cpp:255.
    for name in NICKNAMES_NEVER_ADOPTED {
        let report = report(&MESSAGE.replace("Alpha 1", name));
        assert_eq!(report.nickname_to_adopt(), None, "{name}");
    }
    assert_eq!(report(MESSAGE).nickname_to_adopt(), Some("Alpha 1"));
}
