//! Unit tests for `FREQ`, against `reference/tfar_0_9_12/ts/src/plugin.cpp:1874-1907`,
//! `parseFrequencies` (plugin.cpp:1515-1532) and the message `fn_sendFrequencyInfo.sqf` builds.

use tfar_arma_values::StereoMode;

use super::*;

const FREQUENCY_MESSAGE: &str = "FREQ\t[\"87.6_bluefor|7|0\",\"45.5_bluefor|5|1\",\"bad\"]\t[\"30|9|2\"]\t32.5\ttrue\t20.4\t7.9\tAlpha 1-1\t0.3\t7\t0.8\tignored\t1.5\t25";

fn update(message: &str) -> FrequencyUpdate {
    match GameCommand::parse(message).unwrap() {
        GameCommand::Frequencies(update) => update,
        other => panic!("not a frequency update: {other:?}"),
    }
}

#[test]
fn a_frequency_message_reads_every_stored_field() {
    let update = update(FREQUENCY_MESSAGE);
    let frequencies: Vec<(&str, f64, StereoMode)> = update
        .sw
        .iter()
        .map(|entry| (entry.frequency.as_str(), entry.volume, entry.stereo_mode))
        .collect();
    // "bad" has one part, not three, and is skipped (plugin.cpp:1524).
    assert_eq!(
        frequencies,
        [
            ("87.6_bluefor", 7.0, StereoMode::Both),
            ("45.5_bluefor", 5.0, StereoMode::Left)
        ]
    );
    assert_eq!(update.lr.len(), 1);
    assert_eq!(update.lr[0].stereo_mode, StereoMode::Right);
    assert_eq!(update.dd_frequency, "32.5");
    assert!(update.alive);
    // `parseArmaNumberToInt` rounds (plugin.cpp:1882), `(int)atof` truncates (plugin.cpp:1883).
    assert_eq!(update.my_voice_volume, 20);
    assert_eq!(update.dd_volume_level, 7);
    assert_eq!(update.nickname, "Alpha 1-1");
    assert_eq!(update.waves_level, 0.3);
    assert_eq!(update.terrain_coefficient, 7.0);
    assert_eq!(update.global_volume, 0.8);
    assert_eq!(update.receiving_distance_multiplier, 1.5);
    assert_eq!(update.speaker_distance, 25.0);
}

#[test]
fn field_eleven_is_neither_parsed_nor_checked() {
    // plugin.cpp:1874-1890 never reads tokens[11].
    assert_eq!(update(FREQUENCY_MESSAGE).speaker_distance, 25.0);
}

#[test]
fn alive_is_true_only_for_the_text_true() {
    let message = FREQUENCY_MESSAGE.replace("\ttrue\t", "\tfalse\t");
    assert!(!update(&message).alive);
}

#[test]
fn a_frequency_list_that_is_not_an_array_is_an_error() {
    // F-003: `substr(1, length - 2)` of an empty list throws in the original (plugin.cpp:1517).
    let message = FREQUENCY_MESSAGE.replace("[\"30|9|2\"]", "");
    assert!(matches!(
        GameCommand::parse(&message),
        Err(Error::Malformed {
            command: "FREQ",
            field: "lr",
            ..
        })
    ));
}

#[test]
fn a_malformed_volume_in_an_entry_is_an_error() {
    let message = FREQUENCY_MESSAGE.replace("87.6_bluefor|7|0", "87.6_bluefor|x|0");
    assert!(matches!(
        GameCommand::parse(&message),
        Err(Error::Malformed {
            command: "FREQ",
            field: "sw",
            ..
        })
    ));
}

#[test]
fn every_numeric_field_is_checked() {
    for (from, to, field) in [
        ("\t20.4\t", "\tx\t", "my_voice_volume"),
        ("\t7.9\t", "\tx\t", "dd_volume_level"),
        ("\t0.3\t", "\tx\t", "waves_level"),
        ("\t7\t", "\tx\t", "terrain_coefficient"),
        ("\t0.8\t", "\tx\t", "global_volume"),
        ("\t1.5\t", "\tx\t", "receiving_distance_multiplier"),
        ("\t25", "\tx", "speaker_distance"),
    ] {
        let message = FREQUENCY_MESSAGE.replace(from, to);
        match GameCommand::parse(&message) {
            Err(Error::Malformed {
                command: "FREQ",
                field: found,
                ..
            }) => assert_eq!(found, field),
            other => panic!("{field}: {other:?}"),
        }
    }
}

#[test]
fn a_frequency_message_needs_exactly_fourteen_fields() {
    let short = FREQUENCY_MESSAGE.trim_end_matches("\t25");
    assert!(matches!(
        GameCommand::parse(short),
        Err(Error::WrongFieldCount {
            command: "FREQ",
            expected: 14,
            found: 13
        })
    ));
}
