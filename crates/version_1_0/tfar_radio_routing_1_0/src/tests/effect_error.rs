//! Unit tests for the effect error levels.

use super::*;

fn at(x: f64, y: f64, z: f64) -> Position {
    Position::from_array([x, y, z]).unwrap()
}

#[test]
fn short_and_long_range_errors_are_distance_over_range() {
    assert!(
        (effect_error_from_distance(SendingRadio::ShortRange, 1500.0, 3000) - 0.5).abs() < 1e-12
    );
    assert!((effect_error_from_distance(SendingRadio::LongRange, 300.0, 3000) - 0.1).abs() < 1e-12);
}

#[test]
fn a_diver_radio_error_is_the_distance_itself_as_in_the_original() {
    assert!(
        (effect_error_from_distance(SendingRadio::DiverDigital, 3.0, 3000) - 3.0).abs() < 1e-12
    );
}

#[test]
fn the_antenna_caps_the_error_and_no_antenna_caps_it_at_seven() {
    assert!((capped_by_antenna(0.8, Some(0.3)) - 0.3).abs() < 1e-12);
    assert!((capped_by_antenna(0.2, Some(0.3)) - 0.2).abs() < 1e-12);
    assert!((capped_by_antenna(9.0, None) - 7.0).abs() < 1e-12);
    assert!((capped_by_antenna(0.9, None) - 0.9).abs() < 1e-12);
}

#[test]
fn the_underwater_error_weighs_the_submerged_path() {
    // 200 m fully under water, calm sea, a 3000 m radio: 200 · 3000 / 300 / 3000 = 2/3.
    let error = underwater_effect_error(at(0.0, 0.0, -5.0), at(200.0, 0.0, -5.0), 3000, 0.0, None);
    assert!((error - 200.0 / 300.0).abs() < 1e-9);
}
