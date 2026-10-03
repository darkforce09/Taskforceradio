//! The loudspeaker rule: speaker radios on the sender's frequency.

use std::sync::Arc;

use tfar_client_state_1_0::prelude::{
    ClientData, HeardVia, IgnoreTangents, ReceivedOn, SendingRadio, StereoMode, TangentRadio,
};

use crate::routing_context::RoutingContext;

/// The speakers that play `sender`'s transmission (`clientData.cpp:325-361`): the sender
/// transmits on a short-range radio it can use (tangent or flag) or a long-range one; a speaker
/// on the exact frequency plays it unless it is the sender's own radio, its position is unknown,
/// or it is under water (height below 0). Every speaker carries `antenna_loss`. The receptions
/// are appended to `out` and share their radio ids and vehicles with the session.
pub(crate) fn speaker_receptions(
    context: &RoutingContext<'_>,
    sender: &ClientData,
    ignore: IgnoreTangents,
    antenna_loss: Option<f64>,
    out: &mut Vec<HeardVia>,
) {
    let tangent = sender.transmitting_over;
    let is_short_range = tangent == Some(TangentRadio::ShortRange) || ignore.short_range;
    let is_long_range = tangent == Some(TangentRadio::LongRange) || ignore.long_range;
    if !((sender.can_use_sw_radio && is_short_range) || (sender.can_use_lr_radio && is_long_range))
    {
        return;
    }
    let Some(frequency) = sender.transmitting_frequency.as_deref() else {
        return;
    };
    let over = if is_short_range {
        SendingRadio::ShortRange
    } else {
        SendingRadio::LongRange
    };
    let clients = &context.session.clients;
    let heard = context
        .session
        .game
        .speakers_on(frequency)
        .iter()
        .filter_map(|speaker| {
            let owner = speaker.owner.and_then(|owner| clients.by_id(owner));
            if owner.is_some_and(|owner| owner.client_id() == sender.client_id()) {
                return None;
            }
            let position = speaker
                .position
                .or_else(|| owner.map(|owner| owner.client_position(context.now)))?;
            if position.z() < 0.0 {
                return None;
            }
            Some(HeardVia {
                over,
                on: ReceivedOn::Ground,
                volume: speaker.volume,
                stereo_mode: StereoMode::Both,
                radio_id: Arc::clone(&speaker.radio_id),
                position: Some(position),
                wave_z: speaker.wave_z,
                vehicle: Arc::clone(&speaker.vehicle),
                antenna_loss,
            })
        });
    out.extend(heard);
}
