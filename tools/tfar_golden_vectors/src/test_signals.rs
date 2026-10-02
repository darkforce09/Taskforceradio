//! The deterministic test signals the comparisons feed to the oracle and to the Rust crates.
//!
//! **Role:** builds white noise, an exponential sine sweep, an impulse, an amplitude-modulated
//! noise burst and level steps, plus log-spaced probe frequencies, all as `f64` sequences.
//!
//! **Position:** used by the comparison tests of this crate; nothing else calls it.
//!
//! **Signals and state:** none; pure functions. The noise comes from a seeded xorshift64*
//! generator, so the same seed gives the same samples on every run and every thread.
//!
//! **Invariants:** every sample is finite and lies within `±amplitude`; a signal of `frames`
//! frames holds exactly `frames` samples.

use core::f64::consts::TAU;

#[cfg(test)]
#[path = "tests/test_signals.rs"]
mod tests;

/// The sample rate both mods process voice at, in hertz.
pub const SAMPLE_RATE_HZ: f64 = 48_000.0;

/// A seeded xorshift64* generator of uniform values in `[-1, 1)`.
#[derive(Debug, Clone)]
pub struct NoiseGenerator {
    state: u64,
}

impl NoiseGenerator {
    /// A generator from `seed`; a zero seed is replaced by a fixed non-zero one, since xorshift
    /// stays at zero forever.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            },
        }
    }

    /// The next uniform value in `[-1, 1)`.
    pub fn next_uniform(&mut self) -> f64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        let mixed = self.state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        // The top 53 bits as a fraction in [0, 1), then mapped onto [-1, 1).
        let fraction = u53_to_f64(mixed >> 11) / 9_007_199_254_740_992.0;
        fraction * 2.0 - 1.0
    }
}

/// White noise: `frames` uniform samples within `±amplitude`.
pub fn white_noise(frames: usize, amplitude: f64, seed: u64) -> Vec<f64> {
    let mut generator = NoiseGenerator::new(seed);
    (0..frames)
        .map(|_| amplitude * generator.next_uniform())
        .collect()
}

/// An exponential sine sweep from `start_hz` to `end_hz` over `frames` samples at
/// `sample_rate_hz`, with peak `amplitude`.
pub fn sine_sweep(
    frames: usize,
    start_hz: f64,
    end_hz: f64,
    amplitude: f64,
    sample_rate_hz: f64,
) -> Vec<f64> {
    let duration_s = index_to_f64(frames) / sample_rate_hz;
    let growth = (end_hz / start_hz).ln();
    (0..frames)
        .map(|index| {
            let time_s = index_to_f64(index) / sample_rate_hz;
            let phase =
                TAU * start_hz * duration_s / growth * ((time_s / duration_s * growth).exp() - 1.0);
            amplitude * phase.sin()
        })
        .collect()
}

/// A unit impulse of height `amplitude` at the first of `frames` samples.
pub fn impulse(frames: usize, amplitude: f64) -> Vec<f64> {
    (0..frames)
        .map(|index| if index == 0 { amplitude } else { 0.0 })
        .collect()
}

/// A speech-like burst: white noise within `±amplitude`, amplitude-modulated at `syllable_hz`
/// with full depth, switched on for the middle half of `frames` and at a twentieth of the level
/// outside it, so the compressor sees attacks, holds and releases.
pub fn modulated_noise_burst(
    frames: usize,
    amplitude: f64,
    syllable_hz: f64,
    sample_rate_hz: f64,
    seed: u64,
) -> Vec<f64> {
    let mut generator = NoiseGenerator::new(seed);
    let burst = frames / 4..frames - frames / 4;
    (0..frames)
        .map(|index| {
            let time_s = index_to_f64(index) / sample_rate_hz;
            let modulation = 0.5 + 0.5 * (TAU * syllable_hz * time_s).sin();
            let level = if burst.contains(&index) { 1.0 } else { 0.05 };
            amplitude * level * modulation * generator.next_uniform()
        })
        .collect()
}

/// Level steps: each of `levels` held for `frames_per_level` samples, with the sign alternating
/// every sample so the signal has no DC.
pub fn level_steps(levels: &[f64], frames_per_level: usize) -> Vec<f64> {
    levels
        .iter()
        .flat_map(|level| core::iter::repeat_n(*level, frames_per_level))
        .enumerate()
        .map(|(index, level)| if index % 2 == 0 { level } else { -level })
        .collect()
}

/// `count` frequencies spaced evenly on a log scale from `low_hz` to `high_hz`, both included.
pub fn log_spaced_frequencies(count: usize, low_hz: f64, high_hz: f64) -> Vec<f64> {
    let steps = index_to_f64(count.saturating_sub(1)).max(1.0);
    let ratio = (high_hz / low_hz).ln();
    (0..count)
        .map(|index| low_hz * (ratio * index_to_f64(index) / steps).exp())
        .collect()
}

/// A sample index as `f64`; test signals stay far below 2^53 samples.
#[allow(
    clippy::cast_precision_loss,
    reason = "signal lengths stay far below 2^53, where usize to f64 is exact"
)]
fn index_to_f64(index: usize) -> f64 {
    index as f64
}

/// A value below 2^53 as `f64`, exactly.
#[allow(
    clippy::cast_precision_loss,
    reason = "the value holds at most 53 significant bits, which f64 represents exactly"
)]
fn u53_to_f64(value: u64) -> f64 {
    value as f64
}
