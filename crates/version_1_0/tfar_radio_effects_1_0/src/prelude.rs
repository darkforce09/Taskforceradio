//! The names a caller of `tfar_radio_effects_1_0` imports with `use tfar_radio_effects_1_0::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
pub use crate::effects::{
    AirborneRadioEffect, LongRangeRadioEffect, PersonalRadioEffect, UnderWaterRadioEffect,
};
pub use crate::error_level::error_level_from_distance;
pub use crate::mono_mix_buffer::MonoMixBuffer;
pub use crate::radio_effect::{RadioEffect, SAMPLE_RATE_HZ};
pub use crate::radio_effect_mixdown::process_radio_effect;
pub use tfar_arma_values::StereoMode;
