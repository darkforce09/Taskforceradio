//! Unit tests for `ServerRadioData`, against `SERVER_RADIO_DATA` in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:402-440`.

use super::*;

#[test]
fn a_new_connection_has_the_original_defaults() {
    let server = ServerRadioData::new("Alpha".to_owned());
    assert_eq!(server.my_nickname, "Alpha");
    assert_eq!(server.current_data_frame, INVALID_DATA_FRAME);
    assert!((server.terrain_intersection_coefficient - 7.0).abs() < 1e-12);
    assert!((server.global_volume - 1.0).abs() < 1e-12);
    assert!((server.receiving_distance_multiplier - 1.0).abs() < 1e-12);
    assert!((server.speaker_distance - 20.0).abs() < 1e-12);
    assert!(!server.is_tangent_pressed);
    assert_eq!(server.my_original_nickname, None);
    assert!(server.clients.is_empty());
}

#[test]
fn server_fields_the_original_leaves_uninitialised_start_at_defined_defaults() {
    // `myPosition`, `alive`, `canSpeak`, `ddVolumeLevel`, `myVoiceVolume` and `wavesLevel` are
    // never set by the C++ constructor and read garbage until the first POS or FREQ.
    let server = ServerRadioData::default();
    assert_eq!(server.my_position, Position::ORIGIN);
    assert!(!server.is_alive);
    assert!(server.can_speak);
    assert_eq!(server.dd_volume_level, 0);
    assert_eq!(server.my_voice_volume, 0);
    assert!(server.waves_level.abs() < 1e-12);
}

#[test]
fn server_radio_data_can_live_behind_a_lock_shared_between_threads() {
    // The plugin keeps one `ServerRadioData` per connection behind one mutex, reached from the
    // game link, event, service and audio threads; that needs `Send`.
    fn assert_send<T: Send>() {}
    assert_send::<ServerRadioData>();
    let shared = std::sync::Arc::new(std::sync::Mutex::new(ServerRadioData::default()));
    let writer = std::sync::Arc::clone(&shared);
    std::thread::spawn(move || writer.lock().unwrap().is_alive = true)
        .join()
        .unwrap();
    assert!(shared.lock().unwrap().is_alive);
}

#[test]
fn lookups_by_nickname_find_the_entry_and_its_client_id() {
    let mut server = ServerRadioData::new("Alpha".to_owned());
    server
        .clients
        .insert("Alpha".to_owned(), ClientData::new(ClientId(3)));
    server
        .clients
        .insert("Bravo".to_owned(), ClientData::new(ClientId(4)));
    assert_eq!(server.my_client().unwrap().client_id, ClientId(3));
    assert_eq!(server.client_id_of("Bravo"), Some(ClientId(4)));
    assert_eq!(server.client_id_of("Charlie"), None);
    server.client_mut("Bravo").unwrap().range = 9;
    assert_eq!(server.client("Bravo").unwrap().range, 9);
    server.my_client_mut().unwrap().range = 2;
    assert_eq!(server.client("Alpha").unwrap().range, 2);
}

#[test]
fn a_frequency_without_loudspeakers_has_none() {
    let server = ServerRadioData::default();
    assert_eq!(server.speakers_on("30.1").len(), 0);
}
