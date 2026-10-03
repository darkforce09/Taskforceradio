//! The vehicle intercom rule.

use std::sync::Arc;

use tfar_client_state_1_0::prelude::{
    ClientData, HeardVia, ReceivedOn, SendingRadio, StereoMode, intercom_radio_id,
};

/// The intercom slot of a vehicle without intercom.
const NO_INTERCOM_SLOT: i16 = -1;

/// The volume an intercom reception is listed with (unused by the mix, which takes the
/// `intercomVolume` setting).
const INTERCOM_LISTED_VOLUME: f64 = 7.0;

/// The intercom reception of `sender` by `listener` (`clientData.cpp:283-302`): the intercom is
/// on, the sender holds no radio tangent, and both sit in the same vehicle on the same intercom
/// slot.
pub(crate) fn intercom_reception(
    is_intercom_enabled: bool,
    sender: &ClientData,
    listener: &ClientData,
) -> Option<HeardVia> {
    let vehicle = &sender.vehicle;
    let is_heard = is_intercom_enabled
        && !sender.is_transmitting()
        && vehicle.is_in_vehicle()
        && vehicle.name == listener.vehicle.name
        && vehicle.intercom_slot != NO_INTERCOM_SLOT
        && vehicle.intercom_slot == listener.vehicle.intercom_slot;
    is_heard.then(|| HeardVia {
        over: SendingRadio::ShortRange,
        on: ReceivedOn::Intercom,
        volume: INTERCOM_LISTED_VOLUME,
        stereo_mode: StereoMode::Both,
        radio_id: intercom_radio_id(),
        position: None,
        wave_z: 0.0,
        vehicle: Arc::clone(vehicle),
        antenna_loss: None,
    })
}
