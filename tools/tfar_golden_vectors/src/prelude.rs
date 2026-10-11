//! The names a caller of `tfar_golden_vectors` imports with `use tfar_golden_vectors::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::comparison_metrics::{
    decibel_difference, decibels, max_absolute_difference, relative_rms_error, root_mean_square,
};
pub use crate::oracles::{
    ORACLE_MAX_BUTTERWORTH_ORDER, ORACLE_MAX_CURVE_POINTS, OracleCompressor, OracleCompressorRun,
    OracleCompressorSettings, OracleCone, OracleFilter, OracleFilterDesign, OracleSpatialResult,
    OracleSpatialScene, f3daudio_calculate,
};
pub use crate::test_signals::{
    NoiseGenerator, SAMPLE_RATE_HZ, impulse, level_steps, log_spaced_frequencies,
    modulated_noise_burst, sine_sweep, white_noise,
};
