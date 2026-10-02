//! Unit tests for the diver radio range.

use super::*;

#[test]
fn the_diver_range_runs_from_three_hundred_metres_calm_to_seventy_rough() {
    assert!((diver_radio_distance(0.0) - 300.0).abs() < 1e-12);
    assert!((diver_radio_distance(1.0) - 70.0).abs() < 1e-12);
    assert!((diver_radio_distance(0.5) - 185.0).abs() < 1e-12);
}

#[test]
fn the_underwater_part_is_stretched_to_the_radio_range() {
    // 100 m of 400 m under water, a 3000 m radio, calm sea: 100 · 3000 / 300 + 300.
    let weighted = underwater_weighted_distance(400.0, 100.0, 3000.0, 0.0);
    assert!((weighted - 1300.0).abs() < 1e-9);
}
