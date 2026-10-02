//! Unit tests for `error_level_from_distance` in `tfar_radio_effects_0_9`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "the original formula truncates the scaled error to an index, and table indices are small"
)]

use super::*;

/// `calcErrorLevel` as `reference/tfar_0_9_12/ts/src/RadioEffect.h:152` writes it; `None` where
/// it would read past the table.
fn original_calc_error_level(distance_error: f64) -> Option<f64> {
    let part = (distance_error * 10.0) as usize;
    let from = *ERROR_LEVEL_TABLE.get(part)?;
    let to = *ERROR_LEVEL_TABLE.get(part + 1)?;
    Some(from + (from - to) * (distance_error - part as f64 / 10.0))
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
    assert_eq!(error_level_from_distance(5.0), 0.999);
    assert_eq!(error_level_from_distance(f64::INFINITY), 0.999);
    assert!(
        fine_grid()
            .map(error_level_from_distance)
            .all(|level| (0.0..=0.999).contains(&level))
    );
}

#[test]
fn inside_a_tenth_the_level_runs_linearly_to_the_next_entry() {
    let level = error_level_from_distance(0.15);
    assert!((level - 0.225_000_009).abs() < 1e-9, "{level}");
}

#[test]
fn f_019_error_level_rises_inside_each_tenth_where_the_original_fell() {
    let original_at_start = original_calc_error_level(0.1).unwrap();
    let original_inside = original_calc_error_level(0.15).unwrap();
    assert!(
        original_inside < original_at_start,
        "the original falls inside the tenth: {original_inside} after {original_at_start}"
    );
    assert!((original_inside - 0.142_500_006).abs() < 1e-9);
    let ours = error_level_from_distance(0.15);
    assert!(ours > error_level_from_distance(0.1) && ours < error_level_from_distance(0.2));
    assert!((ours - 0.225_000_009).abs() < 1e-9);
}

#[test]
fn f_019_error_level_stays_inside_the_table_where_the_original_read_past_it() {
    assert_eq!(original_calc_error_level(1.25), None);
    assert_eq!(error_level_from_distance(1.25), 0.999);
}
