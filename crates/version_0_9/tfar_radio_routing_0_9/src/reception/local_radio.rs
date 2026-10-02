//! `is_over_local_radio`: whether the local player hears a sender on one of their own radios.
//!
//! **Role:** the five checks of `isOverLocalRadio`
//! (`reference/tfar_0_9_12/ts/src/plugin.cpp:989-1072`) in their order, a later match
//! overwriting an earlier one.
//!
//! **Position:** called by `is_over_radio`, which adds the loudspeaker radios.
//!
//! **Signals and state:** none; reads the server state.
//!
//! **Invariants:** the check order is short range on the listener's long-range list, short
//! range on short range, long range on short range, long range on long range, then diver; short
//! and long range compare the effective distance with the sender's range, the diver radio the
//! raw distance with its reach; a family counts only when the sender holds it or the ignore flag
//! is set, and both sides can use their radio.

use tfar_client_state_0_9::prelude::{StereoMode, TangentKind};
use tfar_client_state_0_9::{ClientData, FrequencySettings, ServerRadioData};
use tfar_geometry::Position;

use crate::distances::{diver_radio_distance, effective_distance, raw_distance};
use crate::heard_via::{HeardOn, HeardVia, IgnoreTangents, LOCAL_RADIO_ID};

#[cfg(test)]
#[path = "tests/local_radio.rs"]
mod tests;

/// The wave height the local player's own radios carry (plugin.cpp:995).
const OWN_RADIO_WAVE_Z: f64 = 1.0;

/// How `transmitter` reaches `listener`'s own radios, or `None`. `listener` is the local
/// player's entry; their frequency lists come from `server`.
pub fn is_over_local_radio(
    server: &ServerRadioData,
    transmitter: &ClientData,
    listener: &ClientData,
    ignore: IgnoreTangents,
) -> Option<HeardVia> {
    let distance = effective_distance(server, transmitter);
    let is_in_range = distance < f64::from(transmitter.range);
    let frequency = transmitter.frequency.as_str();
    let held = transmitter.tangent;
    let lists = [
        (
            TangentKind::ShortRange,
            HeardOn::LongRange,
            &server.lr_frequencies,
        ),
        (
            TangentKind::ShortRange,
            HeardOn::ShortRange,
            &server.sw_frequencies,
        ),
        (
            TangentKind::LongRange,
            HeardOn::ShortRange,
            &server.sw_frequencies,
        ),
        (
            TangentKind::LongRange,
            HeardOn::LongRange,
            &server.lr_frequencies,
        ),
    ];
    let mut heard: Option<(TangentKind, HeardOn, FrequencySettings)> = None;
    for (over, on, list) in lists {
        if !ignore.is_keyed(over, held) {
            continue;
        }
        let Some(settings) = list.get(frequency) else {
            continue;
        };
        if transmitter.can_use(over) && listener.can_use(listened_family(on)) && is_in_range {
            heard = Some((over, on, *settings));
        }
    }
    if ignore.is_keyed(TangentKind::DiverDigital, held)
        && server.dd_frequency == frequency
        && transmitter.can_use(TangentKind::DiverDigital)
        && listener.can_use(TangentKind::DiverDigital)
        && raw_distance(server, transmitter) < diver_radio_distance(server.waves_level)
    {
        let settings = FrequencySettings {
            volume: server.dd_volume_level,
            stereo_mode: StereoMode::Both,
        };
        heard = Some((TangentKind::DiverDigital, HeardOn::Diver, settings));
    }
    heard.map(|(over, on, settings)| HeardVia {
        over,
        on,
        volume: settings.volume,
        stereo_mode: settings.stereo_mode,
        radio_id: LOCAL_RADIO_ID.to_owned(),
        position: Position::ORIGIN,
        wave_z: OWN_RADIO_WAVE_Z,
        vehicle: listener.vehicle.clone(),
    })
}

/// The radio family the listener needs for a reception on `on`.
fn listened_family(on: HeardOn) -> TangentKind {
    match on {
        HeardOn::ShortRange | HeardOn::Loudspeaker => TangentKind::ShortRange,
        HeardOn::LongRange => TangentKind::LongRange,
        HeardOn::Diver => TangentKind::DiverDigital,
    }
}
