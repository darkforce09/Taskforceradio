//! The carried-radio rule: whether the local player hears a sender on a radio they carry.
//!
//! **Role:** decide the reception of `isOverLocalRadio`
//! (`reference/tfar_1_0/ts/src/clientData.cpp:148-265`): which radio the sender transmits on,
//! whether the frequency is one the listener has on, whether range (stretched under water) or a
//! tower carries it, and on which of the listener's radios it plays.
//!
//! **Position:** a module of the crate's `reception`; called by `is_over_radio` for every
//! sender but the listener; reads the session through a [`RoutingContext`].
//!
//! **Signals and state:** none; a pure function of the context.
//!
//! **Invariants:** a short-range tangent wins over a long-range one; a sender under water
//! without a short-range radio transmits as a diver radio; the long-range radio is preferred
//! when the frequency is on both; in half-duplex mode the radio being transmitted on hears
//! nothing.

use std::sync::Arc;

use crate::antennas::AntennaConnection;
use crate::client_state::prelude::{
    ClientData, FrequencyEntry, HeardVia, IgnoreTangents, ReceivedOn, SendingRadio, TangentRadio,
    local_radio_id,
};
use tfar_foundation::geometry::distance_underwater;

use crate::radio_routing::routing_context::RoutingContext;
use crate::radio_routing::underwater::underwater_weighted_distance;

#[cfg(test)]
#[path = "tests/local_radio.rs"]
mod tests;

/// The wave height listed for a carried radio (`result.waveZ = 1.0f`).
const CARRIED_RADIO_WAVE_Z: f64 = 1.0;

/// The radio `sender` transmits on for the listener, if any: the short-range tangent (or its
/// ignore flag) with a short-range or diver radio, else the long-range tangent (or flag) with a
/// long-range radio (`clientData.cpp:168-181`).
pub(crate) fn sending_radio(
    sender: &ClientData,
    ignore: IgnoreTangents,
    is_sender_underwater: bool,
) -> Option<SendingRadio> {
    let tangent = sender.transmitting_over;
    if (tangent == Some(TangentRadio::ShortRange) || ignore.short_range)
        && (sender.can_use_sw_radio || sender.can_use_dd_radio)
    {
        Some(if is_sender_underwater {
            SendingRadio::DiverDigital
        } else {
            SendingRadio::ShortRange
        })
    } else if (tangent == Some(TangentRadio::LongRange) || ignore.long_range)
        && sender.can_use_lr_radio
    {
        Some(SendingRadio::LongRange)
    } else {
        None
    }
}

/// How the local player hears `sender` on a radio they carry, given the best tower between
/// them, `antenna`; `None` when they do not. The listener is `context.session.clients.own()`.
pub fn is_over_local_radio(
    context: &RoutingContext<'_>,
    sender: &ClientData,
    ignore: IgnoreTangents,
    antenna: Option<&AntennaConnection>,
) -> Option<HeardVia> {
    let session = context.session;
    let game = &session.game;
    let listener = session.clients.own()?;
    let listener_position = listener.client_position(context.now);
    let sender_position = sender.client_position(context.now);
    // Under water with a usable short-range radio means inside a vehicle.
    let is_listener_underwater = listener_position.z() < 0.0 && !listener.can_use_sw_radio;
    let is_sender_underwater = sender_position.z() < 0.0 && !sender.can_use_sw_radio;
    let over = sending_radio(sender, ignore, is_sender_underwater)?;
    let frequency = sender.transmitting_frequency.as_deref()?;
    let long_range_entry = game.lr_frequencies.get(frequency);
    let short_range_entry = game.sw_frequencies.get(frequency);
    if long_range_entry.is_none() && short_range_entry.is_none() {
        return None;
    }
    let range = f64::from(sender.range);
    let mut effective_distance = listener.effective_distance_to(sender, game, context.now);
    if is_listener_underwater || is_sender_underwater {
        effective_distance = underwater_weighted_distance(
            effective_distance,
            distance_underwater(listener_position, sender_position),
            range,
            game.waves_level,
        );
        if effective_distance > range && antenna.is_some() {
            // The tower carries it; only the sender's own reach to the tower is left to check
            // above water.
            effective_distance = 0.0;
        }
    }
    if effective_distance > range {
        let tower = antenna?.antenna().position();
        let to_tower = underwater_weighted_distance(
            sender_position.distance_to(tower),
            distance_underwater(sender_position, tower),
            range,
            game.waves_level,
        );
        if to_tower > range {
            return None;
        }
    }
    let transmitting_radio = if context.settings.full_duplex() {
        None
    } else {
        game.current_transmitting_radio.as_deref()
    };
    let is_blocked = |entry: &FrequencyEntry| {
        transmitting_radio.is_some() && entry.radio_classname.as_deref() == transmitting_radio
    };
    let (on, entry) = match (long_range_entry, short_range_entry) {
        (Some(entry), _) if listener.can_use_lr_radio => (ReceivedOn::LongRange, entry),
        (_, Some(entry)) if listener.can_use_sw_radio => (ReceivedOn::ShortRange, entry),
        _ => return None,
    };
    if is_blocked(entry) {
        return None;
    }
    Some(HeardVia {
        over,
        on,
        volume: entry.volume,
        stereo_mode: entry.stereo_mode,
        radio_id: local_radio_id(),
        position: None,
        wave_z: CARRIED_RADIO_WAVE_Z,
        vehicle: Arc::clone(&listener.vehicle),
        antenna_loss: antenna.map(AntennaConnection::loss),
    })
}
