//! Butterworth and RBJ filters in f64 Direct Form II Transposed, with per-channel state.
//!
//! **Role:** the filter set the radio effects use: Butterworth low-pass and band-pass designed by
//! `sci-rs` as second-order sections, RBJ high-pass and low-pass from `biquad`, all processed in
//! `f64` Direct Form II Transposed with state per channel.
//!
//! **Position:** a module of `tfar_audio`, depending on `tfar_audio::sample_buffer`; used by
//! `tfar_plugin_logic_0_9::client_state`, `tfar_plugin_logic_1_0::client_state`, `tfar_plugin_logic_0_9::radio_effects`,
//! `tfar_plugin_logic_1_0::radio_effects`, `tfar_plugin_logic_0_9::voice_pipeline`, `tfar_plugin_logic_1_0::voice_pipeline`.
//!
//! **Signals and state:** per-channel filter state owned by each [`ChannelFilter`]; no globals.
//! Designing a filter ([`FilterDesign::second_order_sections`], [`ChannelFilter::new`]) allocates
//! and runs off the TeamSpeak audio thread; processing runs on it and allocates nothing.
//!
//! **Invariants:** coefficients are designed once at construction; processing allocates nothing
//! and never panics; a bad design parameter is an [`Error`], never a panic.

mod channel_filter;
mod designs;
mod error;
mod filter_design;
pub mod prelude;
mod second_order_sections;

pub use channel_filter::{ChannelFilter, MonoFilter, StereoFilter};
pub use error::{Error, Result};
pub use filter_design::{FilterDesign, MAX_BUTTERWORTH_ORDER};
pub use second_order_sections::{SecondOrderSection, SecondOrderSections};
