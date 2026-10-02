//! The vocabulary of reception: how a sender transmits and how a listener hears them.
//!
//! **Role:** the value types the routing crate produces and the tangent handling and voice
//! pipeline consume: `HeardVia` (one way a listener hears a sender) and its parts.
//!
//! **Position:** a module of `tfar_client_state_1_0`; filled by `tfar_radio_routing_1_0`
//! (`is_over_radio`), read by [`crate::SessionState::apply_remote_tangent`] and by
//! `tfar_voice_pipeline_1_0`.
//!
//! **Signals and state:** none; value types.
//!
//! **Invariants:** a `HeardVia` always names a real reception (`LISTED_ON_NONE` and
//! `LISTEN_TO_NONE` of `reference/tfar_1_0/ts/src/clientData.hpp:14-28` are never stored); an
//! antenna loss, when present, is below 1.

use tfar_arma_values::StereoMode;
use tfar_game_commands_1_0::VehicleDescriptor;
use tfar_geometry::Position;

#[cfg(test)]
#[path = "tests/reception.rs"]
mod tests;

/// The radio id of a radio the listener carries (`"local_radio"`, `clientData.cpp:184`); a
/// speaker radio carries its own id instead.
pub const LOCAL_RADIO_ID: &str = "local_radio";

/// The radio id of an intercom reception (`"intercom"`), also the key of its effect.
pub const INTERCOM_RADIO_ID: &str = "intercom";

/// The radio a client holds the tangent of: what `TANGENT` (short range, also `TANGENT_DD`)
/// and `TANGENT_LR` set (`currentTransmittingTangentOverType`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TangentRadio {
    /// A short-range (or diver) radio.
    ShortRange,
    /// A long-range radio.
    LongRange,
}

/// The kind of radio a sender's signal comes from (`sendingRadioType`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SendingRadio {
    /// A short-range radio above water.
    ShortRange,
    /// A long-range radio.
    LongRange,
    /// A short-range radio of a sender under water, heard as a diver radio.
    DiverDigital,
}

/// Where the listener hears the signal (`receivingRadioType`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReceivedOn {
    /// A short-range radio the listener carries.
    ShortRange,
    /// A long-range radio the listener carries.
    LongRange,
    /// A radio playing over its loudspeaker, on the ground, in a vehicle or on its owner.
    Ground,
    /// The vehicle intercom.
    Intercom,
}

impl ReceivedOn {
    /// Whether the reception is on a radio the listener carries (short or long range), the
    /// receptions the original lists before `LISTED_ON_NONE`.
    pub const fn is_carried_radio(self) -> bool {
        matches!(self, Self::ShortRange | Self::LongRange)
    }
}

/// The tangent flags `isOverRadio` takes: treat the sender as transmitting on that radio kind
/// whatever their tangent says. The tangent handling sets them to the sender's abilities; every
/// other caller leaves them off.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct IgnoreTangents {
    /// Treat the sender as transmitting on a short-range radio (`ignoreSwTangent`).
    pub short_range: bool,
    /// Treat the sender as transmitting on a long-range radio (`ignoreLrTangent`).
    pub long_range: bool,
}

impl IgnoreTangents {
    /// Both flags off: the sender's tangent decides.
    pub const NONE: Self = Self {
        short_range: false,
        long_range: false,
    };
}

/// One way the listener hears a sender over a radio (`LISTED_INFO`,
/// `reference/tfar_1_0/ts/src/clientData.hpp:30-48`).
#[derive(Clone, Debug, PartialEq)]
pub struct HeardVia {
    /// The kind of radio the sender transmits on.
    pub over: SendingRadio,
    /// Where the listener hears it.
    pub on: ReceivedOn,
    /// The receiving radio's volume (0 to 10 on the radio's knob; 7 for the intercom).
    pub volume: f64,
    /// The ear the receiving radio plays into.
    pub stereo_mode: StereoMode,
    /// [`LOCAL_RADIO_ID`], [`INTERCOM_RADIO_ID`], or the speaker radio's id; the key of the
    /// radio's effects.
    pub radio_id: String,
    /// Where the sound comes from: the speaker's position; `None` for a carried radio and the
    /// intercom, which play at the listener.
    pub position: Option<Position>,
    /// The height that decides whether a speaker plays under water (1 for a carried radio, 0
    /// for the intercom).
    pub wave_z: f64,
    /// The vehicle the sound is in: the listener's for a carried radio, the speaker's for a
    /// speaker, the sender's for the intercom.
    pub vehicle: VehicleDescriptor,
    /// The loss of the best radio tower between sender and listener, if one connects them.
    pub antenna_loss: Option<f64>,
}
