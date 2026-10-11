//! The names a caller of `tfar_plugin_logic_0_9::radio_routing` imports with `use crate::radio_routing::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::radio_routing::distances::{
    diver_radio_distance, effect_error_from_distance, effective_distance, raw_distance,
};
pub use crate::radio_routing::dsp_planning::dsp_requirements;
pub use crate::radio_routing::heard_via::{
    HeardOn, HeardVia, IgnoreTangents, LOCAL_RADIO_ID, local_radio_id,
};
pub use crate::radio_routing::mute_decisions::{
    GameClientMute, MuteDecision, MuteQuery, MuteTracker, game_client_mute,
    set_game_client_mute_status, set_mute_for_dead_players,
};
pub use crate::radio_routing::reception::{
    is_over_local_radio, is_over_radio, is_over_radio_into, receptions,
};
pub use crate::radio_routing::serious_mode::{is_other_radio_plugin_channel, is_serious_mode};
