//! The names a caller of `tfar_game_link_client` imports with `use tfar_game_link_client::prelude::*;`.
//!
//! `Error` and `Result` stay out of the prelude so a glob import never shadows `std`'s `Result`.

pub use crate::client_config::ClientConfig;
pub use crate::crate_role::CRATE_ROLE;
pub use crate::game_link_client::GameLinkClient;
pub use crate::sync_outcome::SyncOutcome;
pub use tfar_game_link_protocol::{Flavour, port};
