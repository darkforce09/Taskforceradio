//! Unit tests for `SpeakerRadio`.

use super::*;
use crate::game_commands::GameCommand;

fn records(message: &str) -> Vec<SpeakerRecord> {
    match GameCommand::parse(message).unwrap() {
        GameCommand::Speakers(records) => records,
        other => panic!("not a SPEAKERS message: {other:?}"),
    }
}

#[test]
fn a_radio_takes_every_field_of_its_record_but_the_frequencies() {
    let record = &records("SPEAKERS\t\u{b}radio_1\n30.1|40.2\nAlpha\n[1,2,3]\n7\n2:5_0.5\n-3")[0];
    let radio = SpeakerRadio::from_record(record);
    assert_eq!(&*radio.radio_id, "radio_1");
    assert_eq!(radio.owner_nickname, "Alpha");
    assert_eq!(radio.position.unwrap().to_array(), [1.0, 2.0, 3.0]);
    assert_eq!(radio.volume, 7);
    assert_eq!(radio.vehicle.vehicle_name, "2:5");
    assert!((radio.wave_z + 3.0).abs() < 1e-12);
}

#[test]
fn a_radio_is_under_water_only_below_the_wave_line() {
    assert!(is_under_water(-1.2));
    assert!(!is_under_water(UNDER_WATER_WAVE_Z));
    assert!(!is_under_water(1.0));
}
