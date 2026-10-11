//! `ClientData`: what the plugin knows about one TeamSpeak client in the game.
//!
//! **Role:** hold every field of `clientData` (`reference/tfar_1_0/ts/src/clientData.hpp:
//! 249-373`) and answer the pure questions about it: where the client is now, how far it is
//! for a radio, whether it is alive.
//!
//! **Position:** a module of `tfar_plugin_logic_1_0::client_state`; owned by [`crate::client_state::ClientDirectory`];
//! written by the session's apply functions; read by the routing crate and the voice pipeline.
//!
//! **Signals and state:** plain data behind the caller's session lock; the game link worker and
//! the TeamSpeak event thread write it, the audio thread reads it and borrows its effects.
//!
//! **Invariants:** the nickname and client id change only through the directory, so its index
//! stays right; every query is pure (time comes in as `now`); a client counts as alive for at
//! most 20 s after its last activity and only within one data frame of the game's.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::game_commands::VehicleDescriptor;
use crate::plugin_commands::TeamspeakClientId;
use tfar_foundation::arma_values::RadioSubtype;
use tfar_foundation::geometry::{Direction, Position};

use crate::client_state::data_frame::DataFrame;
use crate::client_state::effects::ClientEffects;
use crate::client_state::game_data::GameData;
use crate::client_state::receiving_transmissions::ReceivingTransmissions;
use crate::client_state::reception::TangentRadio;

#[cfg(test)]
#[path = "tests/client_data.rs"]
mod tests;

/// How long a client stays alive without activity: five service periods of 4 s
/// (`MILLIS_TO_EXPIRE * 5`, `clientData.cpp:138`).
pub const ALIVE_TIMEOUT: Duration = Duration::from_secs(20);

/// The terrain distance over which terrain interception counts double (`clientData.cpp:130`).
const TERRAIN_INTERCEPTION_DISTANCE: f64 = 2000.0;

/// One TeamSpeak client the plugin knows, in the game or not.
#[derive(Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is one independent field of the original client data"
)]
pub struct ClientData {
    client_id: TeamspeakClientId,
    nickname: String,
    pub(crate) plugin_enabled: bool,
    pub(crate) plugin_enabled_checked_at: Option<Instant>,
    pub(crate) receiving: ReceivingTransmissions,
    /// The radio the client holds the tangent of; `None` when not transmitting.
    pub transmitting_over: Option<TangentRadio>,
    /// The frequency the client transmits on, with the radio code; `None` when not transmitting.
    pub transmitting_frequency: Option<String>,
    /// The subtype of the radio the client last pressed.
    pub transmitting_subtype: Option<RadioSubtype>,
    /// The client's transmission range in metres; 0 when not transmitting.
    pub range: i32,
    /// The client's voice range in metres (`VOLUME`).
    pub voice_volume: i32,
    /// The client talks now (`VOLUME`, or TeamSpeak's talk flag for the local player).
    pub is_talking_now: bool,
    /// The client can speak.
    pub can_speak: bool,
    /// The data frame of the client's last position.
    pub data_frame: DataFrame,
    /// The client can use a short-range radio.
    pub can_use_sw_radio: bool,
    /// The client can use a long-range radio.
    pub can_use_lr_radio: bool,
    /// The client can use a diver radio.
    pub can_use_dd_radio: bool,
    /// The terrain between the client and the local player.
    pub terrain_interception: i32,
    /// The objects between the client and the local player.
    pub object_interception: i32,
    /// The client's voice volume multiplier.
    pub voice_volume_multiplier: f64,
    /// The client is a spectator.
    pub is_spectating: bool,
    /// The client is an enemy of the local player.
    pub is_enemy_to_player: bool,
    /// The client's position at its last update.
    pub position: Position,
    /// The client's view direction.
    pub view_direction: Direction,
    /// When the position was last updated (`POS`); the clock of the extrapolation.
    pub position_updated_at: Option<Instant>,
    /// When the client was last active (`POS` or a tangent); the clock of liveness.
    pub last_activity_at: Option<Instant>,
    /// The vehicle the client is in, with its velocity; shared, so a reception names it without
    /// copying.
    pub vehicle: Arc<VehicleDescriptor>,
    /// The DSP objects of this client as a sender.
    pub effects: ClientEffects,
}

impl ClientData {
    /// A client the plugin has just met: not in the game, not transmitting, in no vehicle.
    pub fn new(client_id: TeamspeakClientId, nickname: impl Into<String>) -> Self {
        Self {
            client_id,
            nickname: nickname.into(),
            plugin_enabled: false,
            plugin_enabled_checked_at: None,
            receiving: ReceivingTransmissions::default(),
            transmitting_over: None,
            transmitting_frequency: None,
            transmitting_subtype: None,
            range: 0,
            voice_volume: 0,
            is_talking_now: false,
            can_speak: true,
            data_frame: DataFrame::INVALID,
            can_use_sw_radio: false,
            can_use_lr_radio: false,
            can_use_dd_radio: false,
            terrain_interception: 0,
            object_interception: 0,
            voice_volume_multiplier: 1.0,
            is_spectating: false,
            is_enemy_to_player: false,
            position: Position::ORIGIN,
            view_direction: Direction::ZERO,
            position_updated_at: None,
            last_activity_at: None,
            vehicle: Arc::new(VehicleDescriptor::not_in_vehicle()),
            effects: ClientEffects::new(),
        }
    }

    /// The client's TeamSpeak id.
    pub const fn client_id(&self) -> TeamspeakClientId {
        self.client_id
    }

    /// The client's nickname, the key the game names it by.
    pub fn nickname(&self) -> &str {
        &self.nickname
    }

    /// Sets the nickname; only the directory calls it, keeping its index right.
    pub(crate) fn set_nickname(&mut self, nickname: String) {
        self.nickname = nickname;
    }

    /// Whether the client holds a radio tangent.
    pub const fn is_transmitting(&self) -> bool {
        self.transmitting_over.is_some()
    }

    /// Where the client is at `now`: the last position moved on by the vehicle velocity for the
    /// time since that position arrived (`getClientPosition`, `clientData.cpp:104-122`).
    pub fn client_position(&self, now: Instant) -> Position {
        let velocity = self.vehicle.velocity;
        let Some(updated_at) = self.position_updated_at else {
            return self.position;
        };
        #[expect(
            clippy::float_cmp,
            reason = "the original tests each component for exactly zero (`Vector3D::isNull`)"
        )]
        let is_standing = velocity == [0.0; 3];
        if is_standing {
            return self.position;
        }
        let elapsed = now.saturating_duration_since(updated_at).as_secs_f64();
        let [x, y, z] = self.position.to_array();
        let [velocity_x, velocity_y, velocity_z] = velocity;
        Position::from_array([
            velocity_x.mul_add(elapsed, x),
            velocity_y.mul_add(elapsed, y),
            velocity_z.mul_add(elapsed, z),
        ])
        .unwrap_or(self.position)
    }

    /// The radio distance from this client (the listener) to `other` (the sender) at `now`:
    /// `(d + T·c + T·c·d/2000) · m` with `T` the sender's terrain interception, `c` the terrain
    /// coefficient and `m` the receiving multiplier (`effectiveDistanceTo`,
    /// `clientData.cpp:125-134`).
    pub fn effective_distance_to(&self, other: &Self, game: &GameData, now: Instant) -> f64 {
        let distance = self
            .client_position(now)
            .distance_to(other.client_position(now));
        let terrain = f64::from(other.terrain_interception) * game.terrain_interception_coefficient;
        let result = terrain.mul_add(distance / TERRAIN_INTERCEPTION_DISTANCE, distance + terrain);
        result * game.receiving_distance_multiplier
    }

    /// Whether the client is alive at `now` in the game's `current_frame`: its frame is valid
    /// and within one of the game's, and it was active within [`ALIVE_TIMEOUT`] (`isAlive`,
    /// `clientData.cpp:136-142`). A pure query: expiry is [`crate::client_state::SessionState::sweep_expired`].
    pub fn is_alive(&self, now: Instant, current_frame: DataFrame) -> bool {
        let is_recent = self
            .last_activity_at
            .is_some_and(|active| now.saturating_duration_since(active) <= ALIVE_TIMEOUT);
        is_recent && self.data_frame.is_within_one_of(current_frame)
    }
}
