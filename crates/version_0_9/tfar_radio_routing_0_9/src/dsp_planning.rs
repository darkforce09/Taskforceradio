//! `dsp_requirements`: the sound state a sender's voice will pass through, from the routing.
//!
//! **Role:** walk the branches of the stereo function
//! (`reference/tfar_0_9_12/ts/src/plugin.cpp:2672-2797`) that pick sound objects, the direct
//! voice and each reception, and name the objects each takes, so the plugin makes them while
//! handling a command and the audio thread finds them.
//!
//! **Position:** called by the plugin after the commands that change a sender's routing (`POS`,
//! `FREQ`, `SPEAKERS`, push-to-talk, `VOLUME`); the result goes to
//! `tfar_client_state_0_9::ClientData::ensure_dsp`.
//!
//! **Signals and state:** none; reads the server state.
//!
//! **Invariants:** the direct voice is named only for another client within voice range; it takes
//! the vehicle low-pass only when it can be heard and a vehicle muffles it, the can't-speak
//! low-pass only when either side cannot speak; every reception names the effect of the sender's
//! subtype, and a loudspeaker adds its band-pass, its vehicle low-pass when muffled and its
//! under-water low-pass below the wave line.

use tfar_client_state_0_9::{
    ClientId, DirectVoiceRequirement, DspRequirements, RadioEffectKind, RadioRequirement,
    ServerRadioData, SpeakerRequirement, VehicleMuffling,
};

use crate::distances::raw_distance;
use crate::error::{Error, Result};
use crate::heard_via::{HeardOn, IgnoreTangents};
use crate::reception::is_over_radio;

#[cfg(test)]
#[path = "tests/dsp_planning.rs"]
mod tests;

/// The sound state the voice of `sender_nickname` needs for the local player
/// `listener_client_id` to hear it.
///
/// # Errors
///
/// [`Error::UnknownSender`] when no entry has the nickname, [`Error::SenderDataNotCurrent`]
/// when the sender's data frame is not current, and [`Error::ListenerDataNotCurrent`] when the
/// listener has no current entry: in each case the voice path passes the voice through or
/// silences it without sound state.
pub fn dsp_requirements(
    server: &ServerRadioData,
    sender_nickname: &str,
    listener_client_id: ClientId,
) -> Result<DspRequirements> {
    let sender = server
        .client(sender_nickname)
        .ok_or_else(|| Error::UnknownSender {
            nickname: sender_nickname.to_owned(),
        })?;
    if !server.is_frame_current(sender.data_frame) {
        return Err(Error::SenderDataNotCurrent {
            nickname: sender_nickname.to_owned(),
        });
    }
    let (_, listener) =
        server
            .fresh_client_by_id(listener_client_id)
            .ok_or(Error::ListenerDataNotCurrent {
                client_id: listener_client_id.get(),
            })?;
    let is_in_voice_range = sender.client_id != listener.client_id
        && raw_distance(server, sender) <= f64::from(sender.voice_volume);
    let direct_voice = is_in_voice_range.then(|| {
        let can_be_heard = sender.can_speak && server.can_speak;
        DirectVoiceRequirement {
            vehicle_isolation: VehicleMuffling::between(&listener.vehicle, &sender.vehicle)
                .muffling_isolation()
                .filter(|_| can_be_heard),
            needs_cant_speak_filter: !can_be_heard,
        }
    });
    let effect = RadioEffectKind::for_subtype(&sender.subtype);
    let radios = is_over_radio(
        server,
        sender_nickname,
        sender,
        listener,
        IgnoreTangents::NONE,
    )
    .into_iter()
    .map(|heard| RadioRequirement {
        speaker: (heard.on == HeardOn::Loudspeaker).then(|| SpeakerRequirement {
            vehicle_isolation: VehicleMuffling::between(&listener.vehicle, &heard.vehicle)
                .muffling_isolation(),
            is_under_water: heard.is_under_water(),
        }),
        radio_id: heard.radio_id,
        effect,
    })
    .collect();
    Ok(DspRequirements {
        direct_voice,
        radios,
    })
}
