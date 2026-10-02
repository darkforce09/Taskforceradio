//! Unit tests for `dsp_requirements`, against the branches of the stereo function in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:2672-2797`.

use super::*;
use tfar_client_state_0_9::prelude::{
    Position, RadioSubtype, StereoMode, TangentKind, UsableRadios, VehicleDescriptor,
};
use tfar_client_state_0_9::{ClientData, FrequencySettings, SpeakerRadio};

const ME: ClientId = ClientId(1);

fn vehicle(name: &str, isolation: f64) -> VehicleDescriptor {
    VehicleDescriptor {
        vehicle_name: name.to_owned(),
        isolation,
    }
}

/// `Me` (1) on foot and `Alpha` (2) 10 m away with a 20 m voice, a `digital` radio held on
/// `30.1`, which the listener has on short range; both current.
fn server() -> ServerRadioData {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.current_data_frame = 4;
    server.sw_frequencies.insert(
        "30.1".to_owned(),
        FrequencySettings {
            volume: 7,
            stereo_mode: StereoMode::Both,
        },
    );
    for (nickname, id, x) in [("Me", 1, 0.0), ("Alpha", 2, 10.0)] {
        let mut client = ClientData::new(ClientId(id));
        client.data_frame = 4;
        client.position = Position::from_array([x, 0.0, 0.0]).unwrap();
        client.voice_volume = 20;
        client.usable_radios = UsableRadios {
            short_range: true,
            long_range: true,
            diver: true,
        };
        client.vehicle = vehicle("no", 0.0);
        server.clients.insert(nickname.to_owned(), client);
    }
    let alpha = server.client_mut("Alpha").unwrap();
    alpha.tangent = Some(TangentKind::ShortRange);
    alpha.frequency = "30.1".to_owned();
    alpha.range = 1000;
    alpha.subtype = RadioSubtype::Digital;
    server
}

#[test]
fn a_sender_in_voice_range_on_a_held_radio_needs_the_voice_and_the_local_radio() {
    let server = server();
    let requirements = dsp_requirements(&server, "Alpha", ME).unwrap();
    assert_eq!(
        requirements.direct_voice,
        Some(DirectVoiceRequirement {
            vehicle_isolation: None,
            needs_cant_speak_filter: false,
        })
    );
    assert_eq!(
        requirements.radios,
        [RadioRequirement {
            radio_id: "local_radio".to_owned(),
            effect: RadioEffectKind::Personal,
            speaker: None,
        }]
    );
}

#[test]
fn beyond_voice_range_or_for_the_listener_themself_there_is_no_direct_voice() {
    let mut server = server();
    server.client_mut("Alpha").unwrap().voice_volume = 5;
    assert_eq!(
        dsp_requirements(&server, "Alpha", ME).unwrap().direct_voice,
        None
    );
    assert_eq!(
        dsp_requirements(&server, "Me", ME).unwrap().direct_voice,
        None
    );
}

#[test]
fn a_vehicle_between_them_needs_the_vehicle_low_pass() {
    let mut server = server();
    server.client_mut("Alpha").unwrap().vehicle = vehicle("1:5", 0.6);
    let voice = dsp_requirements(&server, "Alpha", ME)
        .unwrap()
        .direct_voice
        .unwrap();
    assert_eq!(voice.vehicle_isolation, Some(0.6));
    server.client_mut("Me").unwrap().vehicle = vehicle("1:5", 0.6);
    let voice = dsp_requirements(&server, "Alpha", ME)
        .unwrap()
        .direct_voice
        .unwrap();
    assert_eq!(
        voice.vehicle_isolation, None,
        "the same vehicle does not muffle"
    );
}

#[test]
fn a_sender_who_cannot_speak_needs_the_cant_speak_low_pass_and_no_vehicle_filter() {
    let mut server = server();
    let alpha = server.client_mut("Alpha").unwrap();
    alpha.can_speak = false;
    alpha.vehicle = vehicle("1:5", 0.6);
    let voice = dsp_requirements(&server, "Alpha", ME)
        .unwrap()
        .direct_voice
        .unwrap();
    assert!(voice.needs_cant_speak_filter);
    assert_eq!(voice.vehicle_isolation, None);
    server.client_mut("Alpha").unwrap().can_speak = true;
    server.can_speak = false;
    assert!(
        dsp_requirements(&server, "Alpha", ME)
            .unwrap()
            .direct_voice
            .unwrap()
            .needs_cant_speak_filter,
        "a listener who cannot speak hears muffled too"
    );
}

#[test]
fn the_effect_follows_the_senders_subtype() {
    let mut server = server();
    for (subtype, effect) in [
        (RadioSubtype::DigitalLongRange, RadioEffectKind::LongRange),
        (RadioSubtype::Airborne, RadioEffectKind::LongRange),
        (RadioSubtype::DiverDigital, RadioEffectKind::UnderWater),
        (RadioSubtype::Phone, RadioEffectKind::Phone),
        (
            RadioSubtype::Unknown(String::new()),
            RadioEffectKind::Silent,
        ),
    ] {
        server.client_mut("Alpha").unwrap().subtype = subtype.clone();
        let requirements = dsp_requirements(&server, "Alpha", ME).unwrap();
        assert_eq!(requirements.radios[0].effect, effect, "{subtype:?}");
    }
}

#[test]
fn a_loudspeaker_needs_its_band_pass_vehicle_and_under_water_stages() {
    let mut server = server();
    server.speakers.insert(
        "30.1".to_owned(),
        vec![SpeakerRadio {
            radio_id: "r1".to_owned(),
            owner_nickname: "Me".to_owned(),
            position: Some(Position::from_array([3.0, 0.0, 0.0]).unwrap()),
            volume: 5,
            vehicle: vehicle("2:2", 0.3),
            wave_z: -4.0,
        }],
    );
    let requirements = dsp_requirements(&server, "Alpha", ME).unwrap();
    assert_eq!(
        requirements.radios[1],
        RadioRequirement {
            radio_id: "r1".to_owned(),
            effect: RadioEffectKind::Personal,
            speaker: Some(SpeakerRequirement {
                vehicle_isolation: Some(0.3),
                is_under_water: true,
            }),
        }
    );
}

#[test]
fn a_sender_or_listener_without_current_data_needs_nothing() {
    let mut server = server();
    assert_eq!(
        dsp_requirements(&server, "Nobody", ME),
        Err(Error::UnknownSender {
            nickname: "Nobody".to_owned()
        })
    );
    server.client_mut("Me").unwrap().data_frame = 0;
    assert_eq!(
        dsp_requirements(&server, "Alpha", ME),
        Err(Error::ListenerDataNotCurrent { client_id: 1 })
    );
    server.client_mut("Alpha").unwrap().data_frame = 0;
    assert_eq!(
        dsp_requirements(&server, "Alpha", ME),
        Err(Error::SenderDataNotCurrent {
            nickname: "Alpha".to_owned()
        })
    );
}

#[test]
fn the_requirements_make_every_object_the_voice_path_borrows() {
    let mut server = server();
    let requirements = dsp_requirements(&server, "Alpha", ME).unwrap();
    let alpha = server.client_mut("Alpha").unwrap();
    alpha.ensure_dsp(&requirements, None).unwrap();
    assert!(alpha.dsp.compressor_mut().is_some());
    assert!(
        alpha
            .dsp
            .radio_mut("local_radio")
            .unwrap()
            .personal_effect_mut()
            .is_some()
    );
}
