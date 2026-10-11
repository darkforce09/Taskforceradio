//! Unit tests for the service periods and their test override.

use super::*;

#[test]
fn without_the_variable_the_periods_are_the_originals() {
    let periods = ServicePeriods::from_value(None);
    assert_eq!(periods, ServicePeriods::DEFAULT);
    assert_eq!(periods.tick, Duration::from_millis(100));
    assert_eq!(periods.sweep, Duration::from_secs(4));
    assert_eq!(periods.debug_snapshot, Duration::from_millis(500));
}

#[test]
fn the_default_is_the_default_tick_scaled() {
    assert_eq!(
        ServicePeriods::with_tick(SERVICE_TICK),
        ServicePeriods::DEFAULT
    );
}

#[test]
fn a_shorter_tick_scales_the_sweep_and_the_snapshot() {
    let periods = ServicePeriods::from_value(Some("10"));
    assert_eq!(periods.tick, Duration::from_millis(10));
    assert_eq!(periods.sweep, Duration::from_millis(400));
    assert_eq!(periods.debug_snapshot, Duration::from_millis(50));
    assert_eq!(
        ServicePeriods::from_value(Some(" 1 ")).tick,
        Duration::from_millis(1)
    );
    assert_eq!(
        ServicePeriods::from_value(Some("100")),
        ServicePeriods::DEFAULT
    );
}

#[test]
fn a_value_that_is_not_a_shorter_tick_keeps_the_default() {
    for value in ["0", "101", "1000", "-5", "fast", "", "2.5"] {
        assert_eq!(
            ServicePeriods::from_value(Some(value)),
            ServicePeriods::DEFAULT,
            "{value:?}"
        );
    }
}
