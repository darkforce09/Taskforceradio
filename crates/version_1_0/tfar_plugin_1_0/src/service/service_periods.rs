//! [`ServicePeriods`]: how often the service thread ticks, sweeps and snapshots, and the test-only
//! environment variable that shortens them all together.
//!
//! **Role:** the 100 ms tick, the 4 s sweep and the 500 ms debug snapshot of the original's
//! `ServiceThread` (`reference/tfar_1_0/ts/src/plugin.cpp:133-163`), as one value the service
//! thread starts with.
//!
//! **Position:** read once in `ts3plugin_init` and handed to `ServiceThread::start`.
//!
//! **Signals and state:** none; a plain value.
//!
//! **Invariants:** the sweep is always 40 ticks and the snapshot 5 ticks; the override can only
//! shorten the tick, never lengthen it.

use std::time::Duration;

#[cfg(test)]
#[path = "tests/service_periods.rs"]
mod tests;

/// One tick of the loop.
pub const SERVICE_TICK: Duration = Duration::from_millis(100);

/// The period of the sweep, the channel move and the metadata (`MILLIS_TO_EXPIRE`).
pub const SWEEP_PERIOD: Duration = Duration::from_secs(4);

/// The period of the debug window snapshot.
pub const DEBUG_SNAPSHOT_PERIOD: Duration = Duration::from_millis(500);

/// The ticks in one sweep period.
const TICKS_PER_SWEEP: u32 = 40;

/// The ticks in one debug snapshot period.
const TICKS_PER_DEBUG_SNAPSHOT: u32 = 5;

/// The test-only environment variable that sets the tick in milliseconds, from 1 to 100, and
/// scales the sweep (40 ticks) and the snapshot (5 ticks) with it, so a test sees many seconds of
/// service ticks in a fraction of the time. The game link's timeout and debounce stay in real
/// time. Any other value is ignored with a warning.
pub const SERVICE_TICK_VARIABLE: &str = "TFAR_SERVICE_TICK_MS";

/// The periods of the service thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServicePeriods {
    /// One tick: the game link poll and the release of finished sounds.
    pub tick: Duration,
    /// The sweep, the channel move and the metadata.
    pub sweep: Duration,
    /// The debug window snapshot.
    pub debug_snapshot: Duration,
}

impl ServicePeriods {
    /// The periods of the original: 100 ms, 4 s and 500 ms.
    pub const DEFAULT: Self = Self {
        tick: SERVICE_TICK,
        sweep: SWEEP_PERIOD,
        debug_snapshot: DEBUG_SNAPSHOT_PERIOD,
    };

    /// The periods for a tick of `tick`, the sweep and the snapshot scaled with it.
    pub fn with_tick(tick: Duration) -> Self {
        Self {
            tick,
            sweep: tick.saturating_mul(TICKS_PER_SWEEP),
            debug_snapshot: tick.saturating_mul(TICKS_PER_DEBUG_SNAPSHOT),
        }
    }

    /// The periods a value of [`SERVICE_TICK_VARIABLE`] asks for (`None` when unset); the
    /// default for anything but a whole number of milliseconds from 1 to 100.
    pub fn from_value(value: Option<&str>) -> Self {
        value
            .and_then(parse_tick)
            .map_or(Self::DEFAULT, Self::with_tick)
    }

    /// [`ServicePeriods::from_value`] over the process environment, logging a value that is not
    /// a tick.
    pub fn from_environment() -> Self {
        let value = std::env::var(SERVICE_TICK_VARIABLE).ok();
        if let Some(value) = value.as_deref()
            && parse_tick(value).is_none()
        {
            tracing::warn!(
                variable = SERVICE_TICK_VARIABLE,
                value,
                "the service tick override is not 1 to 100 ms; it is ignored"
            );
        }
        Self::from_value(value.as_deref())
    }
}

impl Default for ServicePeriods {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The tick a value names, or `None` for anything but 1 to 100 whole milliseconds.
fn parse_tick(value: &str) -> Option<Duration> {
    let milliseconds = value.trim().parse::<u64>().ok()?;
    (1..=100)
        .contains(&milliseconds)
        .then(|| Duration::from_millis(milliseconds))
}
