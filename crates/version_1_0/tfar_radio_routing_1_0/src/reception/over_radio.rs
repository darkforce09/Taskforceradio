//! `is_over_radio`: every way the local player hears a sender over radio.

use tfar_antennas_1_0::AntennaConnection;
use tfar_client_state_1_0::prelude::{ClientData, HeardVia, IgnoreTangents};

use crate::reception::intercom::intercom_reception;
use crate::reception::local_radio::is_over_local_radio;
use crate::reception::speakers::speaker_receptions;
use crate::routing_context::RoutingContext;

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
/// Empty without a local player.
pub fn is_over_radio(
    context: &RoutingContext<'_>,
    sender: &ClientData,
    ignore: IgnoreTangents,
) -> Vec<HeardVia> {
    let session = context.session;
    let Some(listener) = session.clients.own() else {
        return Vec::new();
    };
    let mut heard = Vec::new();
    heard.extend(intercom_reception(
        context.settings.intercom_enabled(),
        sender,
        listener,
    ));
    if sender.range == 0 {
        return heard;
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
        heard.extend(is_over_local_radio(
            context,
            sender,
            ignore,
            antenna.as_ref(),
        ));
    }
    let effective_distance = listener.effective_distance_to(sender, &session.game, context.now);
    if effective_distance > range && antenna.is_none() {
        return heard;
    }
    heard.extend(speaker_receptions(
        context,
        sender,
        ignore,
        antenna.as_ref().map(AntennaConnection::loss),
    ));
    heard
}
