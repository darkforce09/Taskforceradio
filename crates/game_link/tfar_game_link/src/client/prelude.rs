//! The names a caller of `tfar_game_link::client` imports with `use crate::client::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::client::client_config::ClientConfig;
pub use crate::client::game_link_client::GameLinkClient;
pub use crate::client::sync_outcome::SyncOutcome;
pub use crate::protocol::{Flavour, port};
