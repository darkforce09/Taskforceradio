//! `is_over_radio`: every way the local player hears a sender's radio, own radios and
//! loudspeakers.
//!
//! **Role:** `isOverRadio` (`reference/tfar_0_9_12/ts/src/plugin.cpp:1091-1158`): the local
//! reception for another sender, then one loudspeaker reception per radio on the sender's
//! frequency that the sender does not carry; `is_over_radio_into`, the same into a reused
//! list; and `receptions`, which finds the two entries first.
//!
//! **Position:** called by the voice pipeline for each frame's routing, by the mute decision and
//! by the plugin for an incoming push-to-talk; uses `is_over_local_radio`.
//!
//! **Signals and state:** none; reads the server state.
//!
//! **Invariants:** the local reception comes first; loudspeaker receptions keep the game's
//! order; a loudspeaker without a position takes its owner's only while the owner's data frame
//! is current, and is left out otherwise; the sender's own loudspeakers are left out by the
//! sender's nickname; filling a list with room for every reception allocates nothing.

use std::sync::Arc;

use tfar_client_state_0_9::prelude::{StereoMode, TangentKind};
use tfar_client_state_0_9::{ClientData, ClientId, ServerRadioData};

use super::local_radio::is_over_local_radio;
use crate::distances::effective_distance;
use crate::heard_via::{HeardOn, HeardVia, IgnoreTangents};

#[cfg(test)]
#[path = "tests/radio_receptions.rs"]
mod tests;

/// Every reception of `transmitter` (the entry of `transmitter_nickname`) by `listener`, the
/// local player's entry: the local reception first, when the two are different clients, then
/// the loudspeakers on the transmitter's frequency while the transmitter is in range and holds
/// (or `ignore` lets through) a short- or long-range push-to-talk it can use.
///
/// Allocates the list; the audio path uses [`is_over_radio_into`] instead.
pub fn is_over_radio(
    server: &ServerRadioData,
    transmitter_nickname: &str,
    transmitter: &ClientData,
    listener: &ClientData,
    ignore: IgnoreTangents,
) -> Vec<HeardVia> {
    let mut heard = Vec::new();
    is_over_radio_into(
        server,
        transmitter_nickname,
        transmitter,
        listener,
        ignore,
        &mut heard,
    );
    heard
}

/// [`is_over_radio`] into `out`, which is cleared first and keeps its capacity: with room for
/// every reception it allocates nothing, since each reception shares its radio id and vehicle
/// with `server`.
pub fn is_over_radio_into(
    server: &ServerRadioData,
    transmitter_nickname: &str,
    transmitter: &ClientData,
    listener: &ClientData,
    ignore: IgnoreTangents,
    out: &mut Vec<HeardVia>,
) {
    out.clear();
    if transmitter.client_id != listener.client_id
        && let Some(local) = is_over_local_radio(server, transmitter, listener, ignore)
    {
        out.push(local);
    }
    let held = transmitter.tangent;
    let keys_short_range = ignore.is_keyed(TangentKind::ShortRange, held);
    let keys_long_range = ignore.is_keyed(TangentKind::LongRange, held);
    let transmits = (transmitter.can_use(TangentKind::ShortRange) && keys_short_range)
        || (transmitter.can_use(TangentKind::LongRange) && keys_long_range);
    if !transmits || effective_distance(server, transmitter) >= f64::from(transmitter.range) {
        return;
    }
    let over = if keys_short_range {
        TangentKind::ShortRange
    } else {
        TangentKind::LongRange
    };
    for speaker in server.speakers_on(&transmitter.frequency) {
        if speaker.owner_nickname == transmitter_nickname {
            continue;
        }
        let position = speaker.position.or_else(|| {
            server
                .client(&speaker.owner_nickname)
                .filter(|owner| server.is_frame_current(owner.data_frame))
                .map(|owner| owner.position)
        });
        if let Some(position) = position {
            out.push(HeardVia {
                over,
                on: HeardOn::Loudspeaker,
                volume: speaker.volume,
                stereo_mode: StereoMode::Both,
                radio_id: Arc::clone(&speaker.radio_id),
                position,
                wave_z: speaker.wave_z,
                vehicle: Arc::clone(&speaker.vehicle),
            });
        }
    }
}

/// The receptions of the player `transmitter_nickname` by the local player `listener_client_id`,
/// with both entries looked up: the transmitter by nickname, the listener by client id with
/// current data. Empty when either is missing, as the original's `isOverRadio` is for a null
/// entry.
pub fn receptions(
    server: &ServerRadioData,
    transmitter_nickname: &str,
    listener_client_id: ClientId,
    ignore: IgnoreTangents,
) -> Vec<HeardVia> {
    match (
        server.client(transmitter_nickname),
        server.fresh_client_by_id(listener_client_id),
    ) {
        (Some(transmitter), Some((_, listener))) => {
            is_over_radio(server, transmitter_nickname, transmitter, listener, ignore)
        }
        _ => Vec::new(),
    }
}
