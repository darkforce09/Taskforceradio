//! Unit tests for `SampleDropout` in `tfar_plugin_logic_1_0::radio_effects`.

#![allow(clippy::cast_precision_loss, reason = "draw counts are far below 2^52")]

use super::*;

/// The share of `draws` samples that `dropout` drops.
fn drop_share(dropout: &mut SampleDropout, draws: u32) -> f64 {
    let dropped = (0..draws).filter(|_| dropout.should_drop()).count();
    dropped as f64 / f64::from(draws)
}

#[test]
fn error_zero_never_drops() {
    let mut dropout = SampleDropout::new(7);
    dropout.set_error_level(0.0);
    assert_eq!(drop_share(&mut dropout, 100_000), 0.0);
}

#[test]
fn the_drop_share_follows_the_error_level() {
    let mut dropout = SampleDropout::new(7);
    for error_level in [0.1_f32, 0.25, 0.5, 0.9] {
        dropout.set_error_level(error_level);
        let share = drop_share(&mut dropout, 200_000);
        assert!(
            (share - f64::from(error_level)).abs() < 0.01,
            "{error_level}: {share}"
        );
    }
}

#[test]
fn error_one_keeps_only_the_top_draw() {
    let mut dropout = SampleDropout::new(7);
    dropout.set_error_level(1.0);
    assert!(drop_share(&mut dropout, 200_000) > 0.999);
}

#[test]
fn the_same_seed_draws_the_same_dropouts() {
    let mut first = SampleDropout::new(42);
    let mut second = SampleDropout::new(42);
    let mut other = SampleDropout::new(43);
    for dropout in [&mut first, &mut second, &mut other] {
        dropout.set_error_level(0.5);
    }
    let first_drops: Vec<bool> = (0..1000).map(|_| first.should_drop()).collect();
    let second_drops: Vec<bool> = (0..1000).map(|_| second.should_drop()).collect();
    let other_drops: Vec<bool> = (0..1000).map(|_| other.should_drop()).collect();
    assert_eq!(first_drops, second_drops);
    assert_ne!(first_drops, other_drops);
}
