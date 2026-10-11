//! Builders the unit tests of `tfar_plugin_logic_1_0::client_state` share.

use std::sync::Arc;
use std::time::Instant;

use crate::game_commands::{FrequencyReport, PositionUpdate, VehicleDescriptor};
use crate::plugin_commands::{TangentBroadcast, TangentKind, TeamspeakClientId};
use tfar_foundation::arma_values::{FrequencyEntry, RadioSubtype, StereoMode};
use tfar_foundation::geometry::{Direction, Position};

use crate::client_state::reception::{HeardVia, ReceivedOn, SendingRadio};
use crate::client_state::session::SessionState;

pub(crate) const OWN: TeamspeakClientId = TeamspeakClientId(1);
pub(crate) const ALPHA: TeamspeakClientId = TeamspeakClientId(2);
pub(crate) const BRAVO: TeamspeakClientId = TeamspeakClientId(3);

pub(crate) fn at(x: f64, y: f64, z: f64) -> Position {
    Position::from_array([x, y, z]).unwrap()
}

/// A session with the local player `me` and the clients `alpha` and `bravo`, in a game.
pub(crate) fn session() -> SessionState {
    let mut session = SessionState::new();
    session.clients.set_own_client(OWN, "me");
    session.clients.client_joined(ALPHA, "alpha");
    session.clients.client_joined(BRAVO, "bravo");
    session.start_game();
    session.game.is_alive = true;
    session
}

pub(crate) fn position_update(nickname: &str, position: Position) -> PositionUpdate {
    PositionUpdate {
        nickname: nickname.to_owned(),
        position,
        view_direction: Direction::from_array([0.0, 1.0, 0.0]).unwrap(),
        can_speak: true,
        can_use_sw_radio: true,
        can_use_lr_radio: true,
        can_use_dd_radio: false,
        vehicle: VehicleDescriptor::not_in_vehicle(),
        terrain_interception: 0,
        voice_volume: 1.0,
        object_interception: 0,
        is_spectating: false,
        is_enemy_to_player: false,
    }
}

pub(crate) fn entry(frequency: &str, volume: f64, classname: &str) -> FrequencyEntry {
    FrequencyEntry {
        frequency: frequency.to_owned(),
        volume,
        stereo_mode: StereoMode::Both,
        radio_classname: Some(classname.to_owned()),
    }
}

pub(crate) fn frequency_report(nickname: &str) -> FrequencyReport {
    FrequencyReport {
        sw_frequencies: vec![entry("100", 7.0, "tf_anprc152")],
        lr_frequencies: vec![entry("40", 5.0, "tf_rt1523g")],
        is_alive: true,
        voice_volume: 20,
        nickname: nickname.to_owned(),
        waves_level: 0.25,
        terrain_interception_coefficient: 7.0,
        global_volume: 0.5,
        receiving_distance_multiplier: 1.0,
        speaker_distance: 20.0,
    }
}

pub(crate) fn tangent(nickname: &str, is_pressed: bool, frequency: &str) -> TangentBroadcast {
    TangentBroadcast {
        kind: TangentKind::ShortRange,
        is_pressed,
        frequency: frequency.to_owned(),
        range: 3000,
        range_text: "3000".to_owned(),
        subtype: RadioSubtype::Digital,
        radio_classname: None,
        can_use_sw_radio: true,
        can_use_dd_radio: false,
        nickname: nickname.to_owned(),
    }
}

pub(crate) fn heard_on(on: ReceivedOn) -> HeardVia {
    HeardVia {
        over: SendingRadio::ShortRange,
        on,
        volume: 7.0,
        stereo_mode: StereoMode::Both,
        radio_id: crate::client_state::reception::local_radio_id(),
        position: None,
        wave_z: 1.0,
        vehicle: Arc::new(VehicleDescriptor::not_in_vehicle()),
        antenna_loss: None,
    }
}

pub(crate) fn now() -> Instant {
    Instant::now()
}
