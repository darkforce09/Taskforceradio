//! `HeardVia`: one way the local player hears a sender's radio, and the flags that route a
//! push-to-talk that is no longer held.
//!
//! **Role:** the result type of the reception rules (`LISTED_INFO`,
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:385`) and the ignore flags they take.
//!
//! **Position:** built by `reception/`, read by the mute decisions, the sound planning, the voice
//! pipeline and the plugin's remote push-to-talk sounds.
//!
//! **Signals and state:** none; value types.
//!
//! **Invariants:** a reception on the listener's own radio carries [`LOCAL_RADIO_ID`], the origin
//! as position and wave height 1; an ignore flag counts its family as held; a reception shares
//! its radio id and vehicle with the server state, so making one copies no text.

use std::sync::{Arc, LazyLock};

use crate::client_state::is_under_water;
use crate::client_state::prelude::{StereoMode, TangentKind, VehicleDescriptor};
use tfar_foundation::geometry::Position;

/// The radio id of the local player's own radios, which keys their sound state
/// (`reference/tfar_0_9_12/ts/src/plugin.cpp:1004`).
pub const LOCAL_RADIO_ID: &str = "local_radio";

/// [`LOCAL_RADIO_ID`] as the shared text every own-radio reception carries; made on first use.
static SHARED_LOCAL_RADIO_ID: LazyLock<Arc<str>> = LazyLock::new(|| Arc::from(LOCAL_RADIO_ID));

/// [`LOCAL_RADIO_ID`] as a shared id: a new handle on one text, so it allocates only on the first
/// call of the process.
pub fn local_radio_id() -> Arc<str> {
    Arc::clone(&SHARED_LOCAL_RADIO_ID)
}

/// Where the local player hears the radio (`LISTED_ON`, plugin.cpp:87).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeardOn {
    /// On one of the local player's short-range radios.
    ShortRange,
    /// On one of the local player's long-range radios.
    LongRange,
    /// On the local player's diver radio.
    Diver,
    /// On a radio playing on loudspeaker (the original's `LISTED_ON_GROUND`).
    Loudspeaker,
}

/// One reception of a sender's radio (`LISTED_INFO`, plugin.cpp:385).
#[derive(Clone, Debug, PartialEq)]
pub struct HeardVia {
    /// The radio family the sender transmits on.
    pub over: TangentKind,
    /// Where the local player hears it.
    pub on: HeardOn,
    /// The receiving radio's volume (the diver volume level for a diver radio).
    pub volume: i32,
    /// The ear the receiving radio plays into; both for a diver radio and a loudspeaker.
    pub stereo_mode: StereoMode,
    /// The radio id that keys the sound state: [`LOCAL_RADIO_ID`] or the loudspeaker radio's,
    /// shared with the server state.
    pub radio_id: Arc<str>,
    /// Where the sound comes from relative to the listener: the origin for the local player's
    /// own radios, the loudspeaker radio's position otherwise.
    pub position: Position,
    /// The receiving radio's height above sea level; 1 for the local player's own radios.
    pub wave_z: f64,
    /// The vehicle the receiving radio is in: the listener's for their own radios; shared with
    /// the server state.
    pub vehicle: Arc<VehicleDescriptor>,
}

impl HeardVia {
    /// Whether the receiving radio sounds under water.
    pub fn is_under_water(&self) -> bool {
        is_under_water(self.wave_z)
    }
}

/// Push-to-talk families to route as if held: an incoming push-to-talk routes its own family
/// this way, so the sound of a release still finds its listeners.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct IgnoreTangents {
    /// Route a short-range transmission whether or not it is held.
    pub short_range: bool,
    /// Route a long-range transmission whether or not it is held.
    pub long_range: bool,
    /// Route a diver radio transmission whether or not it is held.
    pub diver: bool,
}

impl IgnoreTangents {
    /// Route only what is held: the voice path and the mute decision.
    pub const NONE: Self = Self {
        short_range: false,
        long_range: false,
        diver: false,
    };

    /// The flags an incoming push-to-talk of `kind` routes with (plugin.cpp:3012).
    pub const fn for_kind(kind: TangentKind) -> Self {
        Self {
            short_range: matches!(kind, TangentKind::ShortRange),
            long_range: matches!(kind, TangentKind::LongRange),
            diver: matches!(kind, TangentKind::DiverDigital),
        }
    }

    /// Whether the family `kind` counts as held for a sender whose held family is `held`.
    pub fn is_keyed(self, kind: TangentKind, held: Option<TangentKind>) -> bool {
        let is_ignored = match kind {
            TangentKind::ShortRange => self.short_range,
            TangentKind::LongRange => self.long_range,
            TangentKind::DiverDigital => self.diver,
        };
        held == Some(kind) || is_ignored
    }
}
