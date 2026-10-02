//! Unit tests for the `SPEAKERS` parser, against `processSpeakers`
//! (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:493-529`) and the records
//! `fnc_preparePositionCoordinates.sqf:62` and `:74` build.

use crate::error::Error;
use crate::{GameCommand, SpeakerRecord, VehicleDescriptor, parse};

fn records(text: &str) -> Vec<SpeakerRecord> {
    match parse(text).unwrap() {
        GameCommand::Speakers(records) => records,
        other => panic!("not SPEAKERS: {other:?}"),
    }
}

const ON_OWNER: &str = "2:15\n30.1_1|31_1\nAlpha 1\n[]\n7\nno\n12.5";
const ON_GROUND: &str = "tf_anprc152_1\n87.6\nBravo\n[100,200,3]\n5\nno";

#[test]
fn a_speaker_on_its_owner_reads_every_part() {
    let records = records(&format!("SPEAKERS\t{ON_OWNER}"));
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.radio_id, "2:15");
    assert_eq!(record.frequencies, ["30.1_1", "31_1"]);
    assert_eq!(record.nickname, "Alpha 1");
    assert_eq!(record.position, None);
    assert_eq!(record.volume, 7);
    assert_eq!(record.vehicle, VehicleDescriptor::not_in_vehicle());
    assert!((record.wave_z - 12.5).abs() < f64::EPSILON);
}

#[test]
fn the_wave_height_defaults_to_the_position_height_or_one() {
    // CommandProcessor.cpp:520-523.
    let on_ground = records(&format!("SPEAKERS\t{ON_GROUND}"));
    assert_eq!(
        on_ground[0].position.unwrap().to_array(),
        [100.0, 200.0, 3.0]
    );
    assert!((on_ground[0].wave_z - 3.0).abs() < f64::EPSILON);
    let on_owner = ON_OWNER.trim_end_matches("\n12.5");
    let owned = records(&format!("SPEAKERS\t{on_owner}"));
    assert!((owned[0].wave_z - 1.0).abs() < f64::EPSILON);
}

#[test]
fn records_are_split_at_vertical_tab_and_empty_ones_skipped() {
    let records = records(&format!("SPEAKERS\t{ON_OWNER}\x0B\x0B{ON_GROUND}\x0B"));
    assert_eq!(records.len(), 2);
    assert_eq!(records[1].nickname, "Bravo");
}

#[test]
fn a_short_speaker_record_is_skipped_and_the_rest_kept() {
    // The original returns at the first record of fewer than six parts
    // (CommandProcessor.cpp:510), dropping every record after it; here only that record goes.
    let records = records(&format!(
        "SPEAKERS\t{ON_OWNER}\x0Bbroken\npart\x0B{ON_GROUND}"
    ));
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].nickname, "Alpha 1");
    assert_eq!(records[1].nickname, "Bravo");
}

#[test]
fn any_token_count_but_two_is_the_empty_list() {
    // `if (tokens.size() != 2) return;` after clearing the list, CommandProcessor.cpp:496-498.
    assert_eq!(records("SPEAKERS").len(), 0);
    assert_eq!(records(&format!("SPEAKERS\t{ON_OWNER}\textra")).len(), 0);
    assert_eq!(records("SPEAKERS\t").len(), 0);
}

#[test]
fn a_malformed_speaker_part_rejects_the_message() {
    let malformed = ON_GROUND.replace("\n5\n", "\nloud\n");
    assert!(matches!(
        parse(&format!("SPEAKERS\t{malformed}")),
        Err(Error::Malformed {
            field: "speaker volume",
            ..
        })
    ));
}
