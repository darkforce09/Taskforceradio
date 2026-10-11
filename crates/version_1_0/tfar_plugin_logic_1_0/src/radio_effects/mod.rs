//! The 1.0 radio sound chains: filters, compressor input and noise per radio type.
//!
//! **Role:** the 1.0 radio effect chains: the long-range, airborne, personal and underwater
//! effects with the filter and distortion settings of `reference/tfar_1_0/ts/src/RadioEffect.hpp`,
//! the error level that degrades a signal with distance, and the mixdown that runs an effect over
//! a stereo block (`processRadioEffect`, `reference/tfar_1_0/ts/src/RadioEffect.hpp:230`).
//!
//! **Position:** a module of `tfar_plugin_logic_1_0`, depending on `tfar_foundation::arma_values`, `tfar_audio::dsp_filters`,
//! `tfar_audio::sample_buffer`; used by `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_1_0::voice_pipeline`.
//!
//! **Signals and state:** each effect owns its delay line, ring-modulator phase, filter state and,
//! for the underwater effect, its seeded generator; no globals. Effects and the
//! [`MonoMixBuffer`] are created off the TeamSpeak audio thread; [`RadioEffect::process`] and
//! [`process_radio_effect`] run on it and allocate nothing.
//!
//! **Invariants:** the error level rises with distance, never falls, and stays inside its table;
//! processing never panics and never allocates; the processing order of every effect is the
//! original's.

mod effects;
mod error;
mod error_level;
mod mono_mix_buffer;
pub mod prelude;
mod radio_effect;
mod radio_effect_mixdown;
mod signal_stages;

#[cfg(test)]
#[path = "tests/signal_measurements.rs"]
mod signal_measurements;

pub use effects::{
    AirborneRadioEffect, LongRangeRadioEffect, PersonalRadioEffect, UnderWaterRadioEffect,
};
pub use error::{Error, Result};
pub use error_level::error_level_from_distance;
pub use mono_mix_buffer::MonoMixBuffer;
pub use radio_effect::{RadioEffect, SAMPLE_RATE_HZ};
pub use radio_effect_mixdown::process_radio_effect;
