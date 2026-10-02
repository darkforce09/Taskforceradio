//! Unit tests for `error_level_from_distance` in `tfar_radio_effects_1_0`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "the original formula truncates the scaled error to an index, and table indices are small"
)]

use super::*;

/// `calcErrorLevel` as `reference/tfar_1_0/ts/src/RadioEffect.hpp:144` writes it, for a
/// non-negative distance error: the index clamped to `0..=11`, the slope without the ×10.
fn original_calc_error_level(distance_error: f64) -> f64 {
    let part = ((distance_error * 10.0) as usize).min(11);
    let from = ERROR_LEVEL_TABLE[part];
    let to = ERROR_LEVEL_TABLE[part + 1];
    from + (to - from) * (distance_error - part as f64 / 10.0)
}

/// The distance errors from -0.5 to 2.0 in steps of 0.0001.
fn fine_grid() -> impl Iterator<Item = f64> {
    (-5_000..=20_000).map(|step| f64::from(step) / 10_000.0)
}

#[test]
fn every_tenth_hits_its_table_entry() {
    for (index, entry) in ERROR_LEVEL_TABLE.iter().enumerate() {
        let level = error_level_from_distance(index as f64 / 10.0);
        assert!(
            (level - entry).abs() < 1e-9,
            "tenth {index}: {level} against {entry}"
        );
    }
}

#[test]
fn the_level_never_falls_as_the_distance_error_rises() {
    let levels: Vec<f64> = fine_grid().map(error_level_from_distance).collect();
    for pair in levels.windows(2) {
        assert!(pair[1] >= pair[0], "{} then {}", pair[0], pair[1]);
    }
}

#[test]
fn the_level_is_continuous_across_every_tenth() {
    for tenth in 1..=11 {
        let edge = f64::from(tenth) / 10.0;
        let below = error_level_from_distance(edge - 1e-9);
        let above = error_level_from_distance(edge + 1e-9);
        assert!(
            (above - below).abs() < 1e-6,
            "a step of {} at {edge}",
            above - below
        );
    }
}

#[test]
fn the_level_is_clamped_to_the_table() {
    assert_eq!(error_level_from_distance(-1.0), 0.0);
    assert_eq!(error_level_from_distance(f64::NAN), 0.0);
    assert_eq!(error_level_from_distance(f64::NEG_INFINITY), 0.0);
    assert_eq!(error_level_from_distance(5.0), 0.999_99);
    assert_eq!(error_level_from_distance(f64::INFINITY), 0.999_99);
    assert!(
        fine_grid()
            .map(error_level_from_distance)
            .all(|level| (0.0..=0.999_99).contains(&level))
    );
}

#[test]
fn inside_a_tenth_the_level_runs_linearly_to_the_next_entry() {
    let level = error_level_from_distance(0.15);
    assert!((level - 0.225_000_009).abs() < 1e-9, "{level}");
}

#[test]
fn f_011_error_level_slope_spans_each_tenth_where_the_original_covered_a_tenth_of_it() {
    let original_inside = original_calc_error_level(0.15);
    assert!(
        (original_inside - 0.157_500_007).abs() < 1e-9,
        "{original_inside}"
    );
    let ours = error_level_from_distance(0.15);
    assert!((ours - 0.225_000_009).abs() < 1e-9, "{ours}");
}

#[test]
fn f_011_error_level_has_no_step_at_a_tenth_where_the_original_jumped() {
    let original_below = original_calc_error_level(0.2 - 1e-9);
    let original_at = original_calc_error_level(0.2);
    assert!(
        original_at - original_below > 0.13,
        "the original steps from {original_below} to {original_at}"
    );
    let ours_below = error_level_from_distance(0.2 - 1e-9);
    let ours_at = error_level_from_distance(0.2);
    assert!(
        (ours_at - ours_below).abs() < 1e-6,
        "{ours_below} then {ours_at}"
    );
}
