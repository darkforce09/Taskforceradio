//! The names a caller of `tfar_plugin_logic_0_9::radio_effects` imports with `use crate::radio_effects::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::radio_effects::effects::{
    LongRangeRadioEffect, PersonalRadioEffect, UnderWaterRadioEffect,
};
pub use crate::radio_effects::error_level::error_level_from_distance;
pub use crate::radio_effects::mono_mix_buffer::MonoMixBuffer;
pub use crate::radio_effects::radio_effect::{RadioEffect, SAMPLE_RATE_HZ};
pub use crate::radio_effects::radio_effect_mixdown::process_radio_effect;
pub use tfar_foundation::arma_values::StereoMode;
