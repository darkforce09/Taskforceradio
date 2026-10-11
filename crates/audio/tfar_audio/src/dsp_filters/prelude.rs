//! The names a caller of `tfar_audio::dsp_filters` imports with `use crate::dsp_filters::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::dsp_filters::channel_filter::{ChannelFilter, MonoFilter, StereoFilter};
pub use crate::dsp_filters::filter_design::{FilterDesign, MAX_BUTTERWORTH_ORDER};
pub use crate::dsp_filters::second_order_sections::{SecondOrderSection, SecondOrderSections};
