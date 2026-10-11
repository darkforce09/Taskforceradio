//! `mute_decision`: whether serious mode mutes one client.
//!
//! **Role:** decide the TeamSpeak mute of one client from range, radio reception and liveness,
//! as `setGameClientMuteStatus` (`reference/tfar_1_0/ts/src/plugin.cpp:68-94`).
//!
//! **Position:** a module of the crate's `mute`; called by the plugin app after a position,
//! a volume or a tangent; uses [`crate::radio_routing::is_over_radio`].
//!
//! **Signals and state:** none; a pure function of the context.
//!
//! **Invariants:** the local player is never muted; nothing is muted outside serious mode or
//! with `disableAutomaticMute`.

use crate::client_state::prelude::{ClientData, IgnoreTangents};

use crate::radio_routing::reception::is_over_radio;
use crate::radio_routing::routing_context::RoutingContext;

#[cfg(test)]
#[path = "tests/mute_decision.rs"]
mod tests;

/// How far beyond a client's voice range it still counts as heard directly, in metres
/// (`voiceVolume + 15`, `plugin.cpp:81`).
pub const VOICE_RANGE_MARGIN: f64 = 15.0;

/// What `mute_decision` asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MuteDecision {
    /// Mute the client in TeamSpeak (`false`: unmute it).
    pub mute: bool,
    /// Drop the client's voice filters ([`crate::client_state::ClientEffects::reset_voices`]),
    /// as the original does on every mute of a known client.
    pub reset_voices: bool,
}

/// The mute state of `target` (`setGameClientMuteStatus`, `reference/tfar_1_0/ts/src/
/// plugin.cpp:68-94`), `None` when `target` is the local player, who is never muted.
///
/// Outside serious mode, or with `disableAutomaticMute`, the answer is "unmute". In serious mode
/// a client is muted when it is unknown, when the local player is alive and the client is not
/// (unless the local player spectates), or when it is beyond its voice range plus
/// [`VOICE_RANGE_MARGIN`] and not heard over radio. `heard_over_radio` is that last answer when
/// the caller already knows it (the tangent handling); otherwise it is computed with
/// [`is_over_radio`] and no ignore flags.
///
/// `is_serious_mode` is whether the local player is in the serious channel. The original asks
/// whether the target is, which is the same answer for every client the local player can hear,
/// as TeamSpeak plays only the own channel.
pub fn mute_decision(
    context: &RoutingContext<'_>,
    target: Option<&ClientData>,
    is_serious_mode: bool,
    heard_over_radio: Option<bool>,
) -> Option<MuteDecision> {
    let session = context.session;
    let listener = session.clients.own();
    let is_listener = matches!(
        (target, listener),
        (Some(target), Some(listener)) if target.client_id() == listener.client_id()
    );
    if !is_serious_mode || context.settings.disable_automatic_mute() {
        return (!is_listener).then_some(MuteDecision {
            mute: false,
            reset_voices: false,
        });
    }
    // The original compares the two entries, so with neither known it leaves the mute alone.
    if is_listener || (target.is_none() && listener.is_none()) {
        return None;
    }
    let mute = match (target, listener) {
        (Some(target), Some(listener))
            if (session.game.is_alive
                && target.is_alive(context.now, session.game.current_data_frame))
                || listener.is_spectating =>
        {
            let distance = listener
                .client_position(context.now)
                .distance_to(target.client_position(context.now));
            let is_out_of_voice_range =
                distance > f64::from(target.voice_volume) + VOICE_RANGE_MARGIN;
            is_out_of_voice_range
                && !heard_over_radio.unwrap_or_else(|| {
                    !is_over_radio(context, target, IgnoreTangents::NONE).is_empty()
                })
        }
        _ => true,
    };
    Some(MuteDecision {
        mute,
        reset_voices: mute && target.is_some(),
    })
}
