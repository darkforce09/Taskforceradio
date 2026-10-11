//! The names a caller of `tfar_plugin_logic_1_0::radio_routing` imports with `use crate::radio_routing::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::radio_routing::effect_error::{
    capped_by_antenna, effect_error_from_distance, underwater_effect_error,
};
pub use crate::radio_routing::mute::{
    MuteChanges, MuteDecision, MuteTracker, VOICE_RANGE_MARGIN, killed_mute_matrix, mute_decision,
};
pub use crate::radio_routing::reception::{is_over_local_radio, is_over_radio, is_over_radio_into};
pub use crate::radio_routing::remote_tangent::apply_remote_tangent;
pub use crate::radio_routing::routing_context::RoutingContext;
pub use crate::radio_routing::serious_mode::{CHANNEL_ID_PREFIX, ChannelId, serious_mode_channel};
pub use crate::radio_routing::underwater::{diver_radio_distance, underwater_weighted_distance};
pub use tfar_foundation::geometry::distance_underwater;
