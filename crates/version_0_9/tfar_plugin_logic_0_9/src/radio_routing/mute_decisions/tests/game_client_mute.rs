//! Unit tests for the serious-mode mute of one player, against `setGameClientMuteStatus` in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:1188-1218`.

use super::*;
use crate::client_state::prelude::{Position, StereoMode, TangentKind, UsableRadios};
use crate::client_state::{ClientData, DirectVoiceRequirement, DspRequirements, FrequencySettings};

const ME: ClientId = ClientId(1);
const ALPHA: ClientId = ClientId(2);

/// The listener `Me` (client 1, alive) and `Alpha` (client 2) 30 m away with a 20 m voice, not
/// talking, holding no radio; both current.
fn server() -> ServerRadioData {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.current_data_frame = 1;
    server.is_alive = true;
    for (nickname, id, x) in [("Me", ME, 0.0), ("Alpha", ALPHA, 30.0)] {
        let mut client = ClientData::new(id);
        client.data_frame = 1;
        client.position = Position::from_array([x, 0.0, 0.0]).unwrap();
        client.voice_volume = 20;
        client.usable_radios = UsableRadios {
            short_range: true,
            long_range: true,
            diver: true,
        };
        server.clients.insert(nickname.to_owned(), client);
    }
    server
}

fn query(is_in_serious_channel: bool, is_talking_in_teamspeak: bool) -> MuteQuery {
    MuteQuery {
        client_id: ALPHA,
        my_client_id: ME,
        is_in_serious_channel,
        is_talking_in_teamspeak,
    }
}

fn give_alpha_voice_state(server: &mut ServerRadioData) {
    let requirements = DspRequirements {
        direct_voice: Some(DirectVoiceRequirement {
            vehicle_isolation: None,
            needs_cant_speak_filter: true,
        }),
        radios: Vec::new(),
    };
    server
        .client_mut("Alpha")
        .unwrap()
        .ensure_dsp(&requirements, None)
        .unwrap();
}

/// One frame through Alpha's can't-speak low-pass.
fn alpha_cant_speak_frame(server: &mut ServerRadioData) -> [f64; 2] {
    server
        .client_mut("Alpha")
        .unwrap()
        .dsp
        .direct_voice_mut()
        .cant_speak_filter_mut()
        .unwrap()
        .process_frame([0.5, 0.5])
}

fn alpha_has_voice_state(server: &mut ServerRadioData) -> bool {
    !server
        .client_mut("Alpha")
        .unwrap()
        .dsp
        .holds_no_voice_state()
}

#[test]
fn outside_serious_mode_unmute_goes_only_to_clients_this_plugin_muted() {
    // The original sends requestUnmuteClients for the player on every POS outside serious mode.
    let mut server = server();
    let mut tracker = MuteTracker::new();
    for _ in 0..3 {
        assert!(
            set_game_client_mute_status(&mut server, &mut tracker, &query(false, false)).is_empty()
        );
    }
    tracker.decide(ALPHA, true);
    assert_eq!(
        set_game_client_mute_status(&mut server, &mut tracker, &query(false, false)).unmute,
        [ALPHA]
    );
    assert!(
        set_game_client_mute_status(&mut server, &mut tracker, &query(false, false)).is_empty()
    );
}

#[test]
fn in_serious_mode_a_silent_player_out_of_voice_range_is_muted_once_and_has_voice_state_reset() {
    let mut server = server();
    give_alpha_voice_state(&mut server);
    let fresh = alpha_cant_speak_frame(&mut server);
    let _ = alpha_cant_speak_frame(&mut server);
    let mut tracker = MuteTracker::new();
    let decision = set_game_client_mute_status(&mut server, &mut tracker, &query(true, false));
    assert_eq!(decision.mute, [ALPHA]);
    assert!(
        alpha_has_voice_state(&mut server),
        "the reset frees nothing under the server lock"
    );
    assert_eq!(
        alpha_cant_speak_frame(&mut server),
        fresh,
        "the reset cleared the filter memory"
    );
    assert!(set_game_client_mute_status(&mut server, &mut tracker, &query(true, false)).is_empty());
}

#[test]
fn a_talking_player_within_voice_range_stays_unmuted() {
    let mut server = server();
    server.client_mut("Alpha").unwrap().position = Position::from_array([15.0, 0.0, 0.0]).unwrap();
    assert!(
        !game_client_mute(&server, &query(true, true)).should_mute,
        "talking by TeamSpeak's flag"
    );
    server.client_mut("Alpha").unwrap().is_talking_now = true;
    assert!(
        !game_client_mute(&server, &query(true, false)).should_mute,
        "talking by VOLUME"
    );
}

#[test]
fn a_silent_player_within_voice_range_is_muted() {
    let mut server = server();
    server.client_mut("Alpha").unwrap().position = Position::from_array([15.0, 0.0, 0.0]).unwrap();
    assert!(game_client_mute(&server, &query(true, false)).should_mute);
}

#[test]
fn a_talking_player_beyond_voice_range_is_muted() {
    let server = server();
    assert!(game_client_mute(&server, &query(true, true)).should_mute);
}

#[test]
fn a_player_heard_over_a_radio_stays_unmuted_at_any_distance() {
    let mut server = server();
    server.sw_frequencies.insert(
        "30.1".to_owned(),
        FrequencySettings {
            volume: 7,
            stereo_mode: StereoMode::Both,
        },
    );
    let alpha = server.client_mut("Alpha").unwrap();
    alpha.tangent = Some(TangentKind::ShortRange);
    alpha.frequency = "30.1".to_owned();
    alpha.range = 1000;
    assert!(!game_client_mute(&server, &query(true, false)).should_mute);
}

#[test]
fn a_dead_listener_mutes_everyone_in_serious_mode() {
    let mut server = server();
    server.is_alive = false;
    server.client_mut("Alpha").unwrap().is_talking_now = true;
    server.client_mut("Alpha").unwrap().position = Position::ORIGIN;
    assert!(game_client_mute(&server, &query(true, true)).should_mute);
}

#[test]
fn a_missing_entry_is_muted_without_touching_voice_state() {
    // The original calls data->resetVoices() with data null here.
    let mut server = server();
    server.client_mut("Alpha").unwrap().data_frame = 7;
    give_alpha_voice_state(&mut server);
    let wanted = game_client_mute(&server, &query(true, false));
    assert_eq!(
        wanted,
        GameClientMute {
            should_mute: true,
            resets_voices_of: None,
        }
    );
    let mut tracker = MuteTracker::new();
    let decision = set_game_client_mute_status(&mut server, &mut tracker, &query(true, false));
    assert_eq!(decision.mute, [ALPHA]);
    assert!(
        alpha_has_voice_state(&mut server),
        "a stale entry's state is left alone"
    );
    server.clients.remove("Alpha");
    let mut tracker = MuteTracker::new();
    let decision = set_game_client_mute_status(&mut server, &mut tracker, &query(true, false));
    assert_eq!(decision.mute, [ALPHA]);
}

#[test]
fn a_listener_without_current_data_mutes_the_player() {
    let mut server = server();
    server.client_mut("Me").unwrap().data_frame = 9;
    assert!(game_client_mute(&server, &query(true, true)).should_mute);
}
