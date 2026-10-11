//! The original C++ audio code and FAudio's F3DAudio as a live test oracle for the audio crates.
//!
//! **Role:** the test oracle for the audio crates: the build script compiles DSPFilters and
//! SimpleComp straight from `reference/` and the vendored F3DAudio with `cc`; the crate wraps them
//! safely and supplies the test signals and error measures its comparison tests judge
//! `tfar_audio::dsp_filters`, `tfar_audio::dynamics` and `tfar_audio::spatial_audio` by.
//!
//! **Position:** `tools`, tier 0, depending on no workspace crate; its integration tests use the
//! audio crates as dev-dependencies.
//!
//! **Signals and state:** none; every oracle call builds its filter, compressor or F3DAudio handle
//! afresh and drops it before returning.
//!
//! **Invariants:** the oracle code is built only here, never into a deliverable; every call into
//! C++ goes through the one `unsafe` module behind safe wrappers that check buffer shapes.

// The calls into the C++ oracle need `unsafe`; `oracles::oracle_bindings` holds every one of them,
// each behind a safe function that checks the buffer it passes.
#![allow(unsafe_code)]

mod comparison_metrics;
mod error;
mod oracles;
pub mod prelude;
mod test_signals;

pub use comparison_metrics::{
    decibel_difference, decibels, max_absolute_difference, relative_rms_error, root_mean_square,
};
pub use error::{Error, Result};
pub use oracles::{
    ORACLE_MAX_BUTTERWORTH_ORDER, ORACLE_MAX_CURVE_POINTS, OracleCompressor, OracleCompressorRun,
    OracleCompressorSettings, OracleCone, OracleFilter, OracleFilterDesign, OracleSpatialResult,
    OracleSpatialScene, f3daudio_calculate,
};
pub use test_signals::{
    NoiseGenerator, SAMPLE_RATE_HZ, impulse, level_steps, log_spaced_frequencies,
    modulated_noise_burst, sine_sweep, white_noise,
};
