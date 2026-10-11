//! The world the unit tests of `tfar_plugin_logic_1_0::radio_routing` route in.

use std::time::Instant;

use crate::antennas::{Antenna, AntennaManager, NetId};
use crate::client_state::prelude::{
    ClientData, DataFrame, FrequencyEntry, SessionState, StereoMode, TangentRadio,
    TeamspeakClientId,
};
use crate::plugin_settings::{PluginSettings, TypeTag};
use tfar_foundation::geometry::Position;

use crate::radio_routing::routing_context::RoutingContext;

pub(crate) const OWN: TeamspeakClientId = TeamspeakClientId(1);
pub(crate) const ALPHA: TeamspeakClientId = TeamspeakClientId(2);
pub(crate) const BRAVO: TeamspeakClientId = TeamspeakClientId(3);

/// A short-range frequency the listener has on.
pub(crate) const SW_FREQUENCY: &str = "100";
/// A long-range frequency the listener has on.
pub(crate) const LR_FREQUENCY: &str = "40";
/// A frequency on both of the listener's radios.
pub(crate) const SHARED_FREQUENCY: &str = "55";
/// The short-range radio's class name.
pub(crate) const SW_CLASSNAME: &str = "tf_anprc152";

pub(crate) fn at(x: f64, y: f64, z: f64) -> Position {
    Position::from_array([x, y, z]).unwrap()
}

fn entry(frequency: &str, volume: f64, classname: &str) -> (String, FrequencyEntry) {
    (
        frequency.to_owned(),
        FrequencyEntry {
            frequency: frequency.to_owned(),
            volume,
            stereo_mode: StereoMode::Left,
            radio_classname: Some(classname.to_owned()),
        },
    )
}

/// The session, settings and towers a test routes in.
pub(crate) struct World {
    pub(crate) session: SessionState,
    pub(crate) settings: PluginSettings,
    pub(crate) antennas: AntennaManager,
    pub(crate) now: Instant,
}

impl World {
    /// The local player `me` at the origin with both radios, `alpha` and `bravo` alive at the
    /// origin, in a game, on calm water.
    pub(crate) fn new() -> Self {
        let now = Instant::now();
        let mut session = SessionState::new();
        session.clients.set_own_client(OWN, "me");
        session.clients.client_joined(ALPHA, "alpha");
        session.clients.client_joined(BRAVO, "bravo");
        session.start_game();
        session.game.is_alive = true;
        session.game.waves_level = 0.0;
        session.game.sw_frequencies = [
            entry(SW_FREQUENCY, 7.0, SW_CLASSNAME),
            entry(SHARED_FREQUENCY, 6.0, SW_CLASSNAME),
        ]
        .into_iter()
        .collect();
        session.game.lr_frequencies = [
            entry(LR_FREQUENCY, 5.0, "tf_rt1523g"),
            entry(SHARED_FREQUENCY, 4.0, "tf_rt1523g"),
        ]
        .into_iter()
        .collect();
        for client in session.clients.iter_mut() {
            client.data_frame = DataFrame::FIRST;
            client.last_activity_at = Some(now);
            client.position = Position::ORIGIN;
        }
        let own = session.clients.own_mut().unwrap();
        own.can_use_sw_radio = true;
        own.can_use_lr_radio = true;
        Self {
            session,
            settings: PluginSettings::default(),
            antennas: AntennaManager::new(),
            now,
        }
    }

    pub(crate) fn context(&self) -> RoutingContext<'_> {
        RoutingContext {
            session: &self.session,
            settings: &self.settings,
            antennas: &self.antennas,
            now: self.now,
        }
    }

    pub(crate) fn client(&mut self, nickname: &str) -> &mut ClientData {
        self.session.clients.get_mut(nickname).unwrap()
    }

    pub(crate) fn sender(&self, nickname: &str) -> &ClientData {
        self.session.clients.get(nickname).unwrap()
    }

    /// `nickname` holds the tangent of `radio` on `frequency` with `range` metres, at
    /// `position`, able to use the radio it transmits on.
    pub(crate) fn transmit(
        &mut self,
        nickname: &str,
        radio: TangentRadio,
        frequency: &str,
        range: i32,
        position: Position,
    ) {
        let client = self.client(nickname);
        client.transmitting_over = Some(radio);
        client.transmitting_frequency = Some(frequency.to_owned());
        client.range = range;
        client.position = position;
        match radio {
            TangentRadio::ShortRange => client.can_use_sw_radio = true,
            TangentRadio::LongRange => client.can_use_lr_radio = true,
        }
    }

    pub(crate) fn add_tower(&mut self, object: i32, position: Position, range: f64) {
        self.antennas
            .add([Antenna::new(NetId::new(2, object), position, range).unwrap()]);
    }

    pub(crate) fn set_setting(&mut self, key: &str, value: &str) {
        self.settings
            .set(key, value, Some(TypeTag::from_wire("BOOL")))
            .unwrap();
    }
}
