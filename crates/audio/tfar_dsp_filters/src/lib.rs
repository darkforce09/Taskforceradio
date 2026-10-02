//! Butterworth and RBJ filters in f64 Direct Form II Transposed, with per-channel state.
//!
//! **Role:** the filter set the radio effects use: Butterworth low-pass and band-pass designed by
//! `sci-rs` as second-order sections, RBJ high-pass and low-pass from `biquad`, all processed in
//! `f64` Direct Form II Transposed with state per channel.
//!
//! **Position:** `crates/audio`, tier 1, depending on `tfar_sample_buffer`; used by
//! `tfar_client_state_0_9`, `tfar_client_state_1_0`, `tfar_radio_effects_0_9`,
//! `tfar_radio_effects_1_0`, `tfar_voice_pipeline_0_9`, `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** per-channel filter state owned by each [`ChannelFilter`]; no globals.
//! Designing a filter ([`FilterDesign::second_order_sections`], [`ChannelFilter::new`]) allocates
//! and runs off the TeamSpeak audio thread; processing runs on it and allocates nothing.
//!
//! **Invariants:** coefficients are designed once at construction; processing allocates nothing
//! and never panics; a bad design parameter is an [`Error`], never a panic.

mod channel_filter;
mod crate_role;
mod designs;
mod error;
mod filter_design;
pub mod prelude;
mod second_order_sections;

pub use channel_filter::{ChannelFilter, MonoFilter, StereoFilter};
pub use crate_role::CRATE_ROLE;
pub use error::{Error, Result};
pub use filter_design::{FilterDesign, MAX_BUTTERWORTH_ORDER};
pub use second_order_sections::{SecondOrderSection, SecondOrderSections};
