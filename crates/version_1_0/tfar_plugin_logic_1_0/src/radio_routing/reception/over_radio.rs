//! `is_over_radio`: every way the local player hears a sender over radio.
//!
//! **Role:** `isOverRadio` (`reference/tfar_1_0/ts/src/clientData.cpp:267-364`): the intercom,
//! the carried radio with the best tower, and the speakers, in the original's order; and
//! `is_over_radio_into`, the same into a list the caller reuses.
//!
//! **Position:** a module of the crate's `reception`; called by the voice pipeline for each
//! frame, by the mute decision and by `apply_remote_tangent`; combines the intercom,
//! carried-radio and speaker rules.
//!
//! **Signals and state:** none; reads the session through a [`RoutingContext`].
//!
//! **Invariants:** the intercom comes first; without range, or beyond it without a tower, only
//! the intercom is heard; the listener never hears itself on a carried radio; a list with room
//! for every reception is refilled without allocating, each reception sharing its radio id and
//! vehicle with the session.

use crate::antennas::AntennaConnection;
use crate::client_state::prelude::{ClientData, HeardVia, IgnoreTangents};

use crate::radio_routing::reception::intercom::intercom_reception;
use crate::radio_routing::reception::local_radio::is_over_local_radio;
use crate::radio_routing::reception::speakers::speaker_receptions;
use crate::radio_routing::routing_context::RoutingContext;

#[cfg(test)]
#[path = "tests/over_radio.rs"]
mod tests;

/// Every way the local player hears `sender` over radio (`isOverRadio`,
/// `reference/tfar_1_0/ts/src/clientData.cpp:267-364`), in the original's order:
///
/// 1. the intercom, checked before anything else;
/// 2. nothing more when the sender has no range (holds no tangent);
/// 3. a radio the listener carries, unless the sender is the listener, with the best tower
///    between them;
/// 4. nothing more when the sender's effective distance exceeds its range and no tower helps;
/// 5. the speakers on the sender's frequency.
///
/// Empty without a local player. Allocates the list; the audio path uses
/// [`is_over_radio_into`] instead.
pub fn is_over_radio(
    context: &RoutingContext<'_>,
    sender: &ClientData,
    ignore: IgnoreTangents,
) -> Vec<HeardVia> {
    let mut heard = Vec::new();
    is_over_radio_into(context, sender, ignore, &mut heard);
    heard
}

/// [`is_over_radio`] into `out`, which is cleared first and keeps its capacity: with room for
/// every reception it allocates nothing, since each reception shares its radio id and vehicle
/// with the session.
pub fn is_over_radio_into(
    context: &RoutingContext<'_>,
    sender: &ClientData,
    ignore: IgnoreTangents,
    out: &mut Vec<HeardVia>,
) {
    out.clear();
    let session = context.session;
    let Some(listener) = session.clients.own() else {
        return;
    };
    out.extend(intercom_reception(
        context.settings.intercom_enabled(),
        sender,
        listener,
    ));
    if sender.range == 0 {
        return;
    }
    let range = f64::from(sender.range);
    let is_listener = sender.client_id() == listener.client_id();
    let antenna = if is_listener {
        None
    } else {
        context.antennas.find_connection(
            sender.client_position(context.now),
            range,
            listener.client_position(context.now),
        )
    };
    if !is_listener {
        out.extend(is_over_local_radio(
            context,
            sender,
            ignore,
            antenna.as_ref(),
        ));
    }
    let effective_distance = listener.effective_distance_to(sender, &session.game, context.now);
    if effective_distance > range && antenna.is_none() {
        return;
    }
    speaker_receptions(
        context,
        sender,
        ignore,
        antenna.as_ref().map(AntennaConnection::loss),
        out,
    );
}
