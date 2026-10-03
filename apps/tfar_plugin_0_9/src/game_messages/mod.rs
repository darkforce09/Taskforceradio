//! The game's messages over the game link: the bridge from the link server, the dispatch, and
//! the position, push-to-talk and frequency messages.

mod frequency_message;
mod game_link_bridge;
mod local_tangent_message;
mod message_dispatch;
mod position_message;

pub use game_link_bridge::GameLinkBridge;
pub use local_tangent_message::{command_tangent_kind, game_tangent_kind};
pub use message_dispatch::speaking_reply;
