//! `ClientData`: what the 0.9.12 plugin knows about one player, keyed by nickname.
//!
//! **Role:** hold every field of the original's `CLIENT_DATA`
//! (`reference/tfar_0_9_12/ts/src/plugin.cpp:192-366`) and the player's sound state, with the
//! two resets, the plugin-enabled cache and the request to make sound state.
//!
//! **Position:** the value type of `ServerRadioData::clients`; written by the command
//! application, read by the routing and the voice pipeline.
//!
//! **Signals and state:** plain fields plus the `ClientDsp`. It lives inside the plugin's server
//! lock; the audio thread reaches it only through that lock (with `try_lock`).
//!
//! **Invariants:** every field starts at a defined value, including the plugin-enabled flag,
//! its check time and the view angle, which the original leaves uninitialised; the client id is
//! set when the entry is made, so an entry always names a TeamSpeak client.

use std::time::{Duration, Instant};

use tfar_game_commands_0_9::prelude::RadioSubtype;
use tfar_game_commands_0_9::{TangentKind, UsableRadios, VehicleDescriptor};
use tfar_geometry::{Angle, Position};
use tfar_hrtf::HrirLibrary;

use crate::client_id::ClientId;
use crate::dsp_state::{ClientDsp, DspRequirements};
use crate::error::Result;

#[cfg(test)]
#[path = "tests/client_data.rs"]
mod tests;

/// How long a positive plugin check stands in for a failed one
/// (`isPluginEnabledForUser`, `reference/tfar_0_9_12/ts/src/plugin.cpp:2545`).
pub const PLUGIN_ENABLED_GRACE: Duration = Duration::from_secs(10);

/// One player as the 0.9.12 plugin tracks them.
#[derive(Debug)]
pub struct ClientData {
    /// The player's TeamSpeak client id, from the channel client list.
    pub client_id: ClientId,
    /// Whether the player runs the plugin, from their plugin commands and metadata.
    pub is_plugin_enabled: bool,
    /// When the plugin was last seen running for the player; `None` before the first sign.
    pub plugin_enabled_check: Option<Instant>,
    /// The radio family whose push-to-talk the player holds; `None` when none is held.
    pub tangent: Option<TangentKind>,
    /// The player's position relative to the local listener's eyes (`POS` fields 2 to 4).
    pub position: Position,
    /// When the last `POS` or push-to-talk of the player arrived; `None` before the first.
    pub position_time: Option<Instant>,
    /// The frequency of the player's last push-to-talk, radio code appended.
    pub frequency: String,
    /// The distance in metres the player's voice carries (`VOLUME`).
    pub voice_volume: i32,
    /// The range in metres of the radio the player last keyed.
    pub range: i32,
    /// Whether the player can speak (not under water, not in an isolated vehicle).
    pub can_speak: bool,
    /// Whether the player talks now, by their last `VOLUME`.
    pub is_talking_now: bool,
    /// The data frame of the player's last `POS`.
    pub data_frame: i32,
    /// Which radio families the player can use.
    pub usable_radios: UsableRadios,
    /// The subtype of the radio the player last keyed.
    pub subtype: RadioSubtype,
    /// The vehicle the player sits in.
    pub vehicle: VehicleDescriptor,
    /// How much terrain lies between the player and the local listener.
    pub terrain_interception: i32,
    /// The direction the player looks; kept for the local player only, where it turns the
    /// direction of every sound.
    pub view_angle: Angle,
    /// The player's voice volume multiplier.
    pub voice_volume_multiplier: f64,
    /// The sound state the player's voice passes through.
    pub dsp: ClientDsp,
}

impl ClientData {
    /// A player with client id `client_id` and the original's defaults: can speak, no radio
    /// usable, no push-to-talk, at the origin, data frame 0, multiplier 1.
    pub fn new(client_id: ClientId) -> Self {
        Self {
            client_id,
            is_plugin_enabled: false,
            plugin_enabled_check: None,
            tangent: None,
            position: Position::ORIGIN,
            position_time: None,
            frequency: String::new(),
            voice_volume: 0,
            range: 0,
            can_speak: true,
            is_talking_now: false,
            data_frame: 0,
            usable_radios: UsableRadios {
                short_range: false,
                long_range: false,
                diver: false,
            },
            subtype: RadioSubtype::Unknown(String::new()),
            vehicle: VehicleDescriptor {
                vehicle_name: String::new(),
                isolation: 0.0,
            },
            terrain_interception: 0,
            view_angle: Angle::ZERO,
            voice_volume_multiplier: 1.0,
            dsp: ClientDsp::default(),
        }
    }

    /// Whether the player holds a push-to-talk or talks by their last `VOLUME`: the radio half
    /// of the `SPEAKING` reply.
    pub fn is_talking_on_radio(&self) -> bool {
        self.tangent.is_some() || self.is_talking_now
    }

    /// Whether the player can use the radio family `kind`.
    pub fn can_use(&self, kind: TangentKind) -> bool {
        match kind {
            TangentKind::ShortRange => self.usable_radios.short_range,
            TangentKind::LongRange => self.usable_radios.long_range,
            TangentKind::DiverDigital => self.usable_radios.diver,
        }
    }

    /// Marks the player able to use the radio family `kind`, as a push-to-talk does.
    pub fn allow_radio(&mut self, kind: TangentKind) {
        match kind {
            TangentKind::ShortRange => self.usable_radios.short_range = true,
            TangentKind::LongRange => self.usable_radios.long_range = true,
            TangentKind::DiverDigital => self.usable_radios.diver = true,
        }
    }

    /// Records a metadata check and returns whether the plugin counts as enabled
    /// (`isPluginEnabledForUser`, plugin.cpp:2526): a positive check stamps the time; a
    /// negative one keeps the last state while the last positive sign is younger than
    /// [`PLUGIN_ENABLED_GRACE`]. The result becomes the stored state.
    pub fn refresh_plugin_enabled(&mut self, metadata_shows_enabled: bool, now: Instant) -> bool {
        let is_enabled = if metadata_shows_enabled {
            self.plugin_enabled_check = Some(now);
            true
        } else {
            self.plugin_enabled_check.is_some_and(|checked| {
                now.saturating_duration_since(checked) < PLUGIN_ENABLED_GRACE
            }) && self.is_plugin_enabled
        };
        self.is_plugin_enabled = is_enabled;
        is_enabled
    }

    /// Makes the sound state `requirements` names; direction sources only when `hrtf` is
    /// given. Call it while handling a command, never on the audio thread.
    ///
    /// # Errors
    ///
    /// [`crate::Error`] when a filter, effect or the compressor cannot be built.
    pub fn ensure_dsp(
        &mut self,
        requirements: &DspRequirements,
        hrtf: Option<&HrirLibrary>,
    ) -> Result<()> {
        self.dsp.ensure(requirements, hrtf)
    }

    /// Drops the radio effects and the loudspeaker and phone filters (`resetRadioEffect`).
    pub fn reset_radio_effects(&mut self) {
        self.dsp.reset_radio_effects();
    }

    /// Drops the direction sources and the can't-speak and vehicle filters (`resetVoices`).
    pub fn reset_voices(&mut self) {
        self.dsp.reset_voices();
    }
}
