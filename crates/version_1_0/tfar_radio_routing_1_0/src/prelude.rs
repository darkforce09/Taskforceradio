//! The names a caller of `tfar_radio_routing_1_0` imports with `use tfar_radio_routing_1_0::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::crate_role::CRATE_ROLE;
pub use crate::effect_error::{
    capped_by_antenna, effect_error_from_distance, underwater_effect_error,
};
pub use crate::mute::{
    MuteChanges, MuteDecision, MuteTracker, VOICE_RANGE_MARGIN, killed_mute_matrix, mute_decision,
};
pub use crate::reception::{is_over_local_radio, is_over_radio};
pub use crate::remote_tangent::apply_remote_tangent;
pub use crate::routing_context::RoutingContext;
pub use crate::serious_mode::{CHANNEL_ID_PREFIX, ChannelId, serious_mode_channel};
pub use crate::underwater::{diver_radio_distance, underwater_weighted_distance};
pub use tfar_geometry::distance_underwater;
