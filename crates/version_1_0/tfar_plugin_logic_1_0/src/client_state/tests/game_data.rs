//! Unit tests for `GameData`.

use super::*;
use crate::client_state::test_support::{entry, frequency_report};

#[test]
fn the_defaults_are_the_originals() {
    let game = GameData::default();
    assert_eq!(game.voice_volume, 0);
    assert!(!game.is_alive);
    assert!((game.terrain_interception_coefficient - 7.0).abs() < f64::EPSILON);
    assert!((game.global_volume - 1.0).abs() < f64::EPSILON);
    assert!((game.receiving_distance_multiplier - 1.0).abs() < f64::EPSILON);
    assert!((game.speaker_distance - 20.0).abs() < f64::EPSILON);
    assert_eq!(game.current_data_frame, DataFrame::INVALID);
    assert_eq!(game.current_transmitting_radio, None);
}

#[test]
fn a_frequency_report_sets_every_value_and_a_later_duplicate_wins() {
    let mut game = GameData::default();
    let mut report = frequency_report("me");
    report
        .sw_frequencies
        .push(entry("100", 3.0, "tf_rf7800str"));
    game.apply_frequency_report(&report);
    let short_range = game.sw_frequencies.get("100").unwrap();
    assert_eq!(short_range.radio_classname.as_deref(), Some("tf_rf7800str"));
    assert!(game.lr_frequencies.contains_key("40"));
    assert!(game.is_alive);
    assert_eq!(game.voice_volume, 20);
    assert!((game.waves_level - 0.25).abs() < f64::EPSILON);
    assert!((game.global_volume - 0.5).abs() < f64::EPSILON);
}

#[test]
fn a_game_starts_at_the_first_frame_and_ends_dead_and_invalid() {
    let mut game = GameData::default();
    game.next_data_frame();
    assert_eq!(game.current_data_frame, DataFrame::INVALID);
    game.start_game();
    game.next_data_frame();
    assert_eq!(game.current_data_frame, DataFrame::new(1));
    game.is_alive = true;
    game.end_game();
    assert!(!game.is_alive);
    assert_eq!(game.current_data_frame, DataFrame::INVALID);
}

#[test]
fn speakers_on_an_unknown_frequency_are_none() {
    assert_eq!(GameData::default().speakers_on("100"), []);
}
