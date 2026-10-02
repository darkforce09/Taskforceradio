//! Unit tests for `SPEAKERS`, against `processSpeakers` in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:1728-1767` and the message
//! `fn_processPlayerPositions.sqf` and `fn_preparePositionCoordinates.sqf` build.

use super::*;

/// A record as the game writes it: parts joined by line feeds.
fn record(parts: &[&str]) -> String {
    parts.join("\n")
}

/// A `SPEAKERS` message as the game writes it: a vertical tab before every record.
fn message(records: &[String]) -> String {
    let mut text = "SPEAKERS\t".to_owned();
    for record in records {
        text.push('\x0B');
        text.push_str(record);
    }
    text
}

fn speakers(text: &str) -> Vec<SpeakerRecord> {
    match GameCommand::parse(text).unwrap() {
        GameCommand::Speakers(speakers) => speakers,
        other => panic!("not a speaker list: {other:?}"),
    }
}

#[test]
fn no_speakers_is_an_empty_list() {
    // The game sends "SPEAKERS\t" with no record; getline drops the trailing empty field.
    assert_eq!(speakers("SPEAKERS\t"), Vec::new());
    assert_eq!(speakers("SPEAKERS"), Vec::new());
}

#[test]
fn more_than_two_fields_is_an_empty_list_that_clears_the_speakers() {
    // plugin.cpp:1731-1732 clears, then reads records only when `tokens.size() == 2`.
    assert_eq!(speakers("SPEAKERS\ta\tb"), Vec::new());
}

#[test]
fn records_are_read_and_the_empty_first_record_skipped() {
    let carried = record(&[
        "2:77",
        "87.6_bluefor|45_bluefor",
        "Alpha",
        "[]",
        "7",
        "no",
        "12.5",
    ]);
    let placed = record(&[
        "tf_rf7800str_1",
        "60",
        "Bravo",
        "[1,2,3]",
        "5.5",
        "2:345_0.5",
    ]);
    let speakers = speakers(&message(&[carried, placed]));
    assert_eq!(
        speakers,
        [
            SpeakerRecord {
                radio_id: "2:77".to_owned(),
                frequencies: vec!["87.6_bluefor".to_owned(), "45_bluefor".to_owned()],
                owner_nickname: "Alpha".to_owned(),
                position: None,
                volume: 7,
                vehicle: VehicleDescriptor {
                    vehicle_name: "no".to_owned(),
                    isolation: 0.0
                },
                wave_z: 12.5,
            },
            SpeakerRecord {
                radio_id: "tf_rf7800str_1".to_owned(),
                frequencies: vec!["60".to_owned()],
                owner_nickname: "Bravo".to_owned(),
                position: Some(Position::from_array([1.0, 2.0, 3.0]).unwrap()),
                // `parseArmaNumberToInt` rounds, plugin.cpp:1752.
                volume: 6,
                vehicle: VehicleDescriptor {
                    vehicle_name: "2:345".to_owned(),
                    isolation: 0.5
                },
                // No seventh part: waveZ is 1, plugin.cpp:1756-1757.
                wave_z: 1.0,
            },
        ]
    );
}

#[test]
fn a_position_of_two_characters_or_fewer_is_none() {
    // `coordinates.length() > 2` gates the read, plugin.cpp:1745.
    let short = record(&["r", "60", "A", "", "5", "no"]);
    assert_eq!(speakers(&message(&[short]))[0].position, None);
}

#[test]
fn a_record_of_fewer_than_six_parts_is_an_error_where_the_original_reads_out_of_bounds() {
    // F-003: plugin.cpp:1740-1753 indexes parts[0..5] without a count check.
    let short = record(&["r", "60", "A", "[]", "5"]);
    assert_eq!(
        GameCommand::parse(&message(&[short])).unwrap_err(),
        Error::ShortSpeakerRecord {
            record: 1,
            parts: 5
        }
    );
}

#[test]
fn a_malformed_part_names_its_field() {
    for (parts, field) in [
        (["r", "60", "A", "[1,2]", "5", "no", "1"], "position"),
        (["r", "60", "A", "[]", "loud", "no", "1"], "volume"),
        (["r", "60", "A", "[]", "5", "v_x", "1"], "vehicle_id"),
        (["r", "60", "A", "[]", "5", "no", "deep"], "wave_z"),
    ] {
        match GameCommand::parse(&message(&[record(&parts)])) {
            Err(Error::Malformed {
                command: "SPEAKERS",
                field: found,
                ..
            }) => assert_eq!(found, field),
            other => panic!("{field}: {other:?}"),
        }
    }
}
