//! The mute decisions: which clients the plugin mutes in TeamSpeak, and the requests for it.

mod dead_players;
mod game_client_mute;
mod mute_tracker;

pub use dead_players::set_mute_for_dead_players;
pub use game_client_mute::{
    GameClientMute, MuteQuery, game_client_mute, set_game_client_mute_status,
};
pub use mute_tracker::{MuteDecision, MuteTracker};
