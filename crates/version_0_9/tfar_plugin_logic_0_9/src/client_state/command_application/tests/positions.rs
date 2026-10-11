//! Unit tests for `POS` and `IS_SPEAKING`, against `processUnitPosition` and the `IS_SPEAKING`
//! branch in `reference/tfar_0_9_12/ts/src/plugin.cpp:1580-1678, 1909-1934`.

use super::*;
use crate::game_commands::GameCommand;
use crate::game_commands::TangentKind;
use tfar_foundation::geometry::{Angle, Position};

fn position(nickname: &str, xyz: [f64; 3], view: f64, direction: f64) -> PositionUpdate {
    let [x, y, z] = xyz;
    let message = format!(
        "POS\t{nickname}\t{x}\t{y}\t{z}\t{view}\ttrue\ttrue\tfalse\ttrue\t1:2_0.5\t3\t0.8\t{direction}"
    );
    match GameCommand::parse(&message).unwrap() {
        GameCommand::Position(update) => update,
        other => panic!("not a POS: {other:?}"),
    }
}

fn server() -> ServerRadioData {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.update_nicknames(&[
        (ClientId(1), "Me".to_owned()),
        (ClientId(2), "Alpha".to_owned()),
    ]);
    server.current_data_frame = 6;
    server
}

#[test]
fn the_local_players_pos_updates_their_entry_view_and_position() {
    let mut server = server();
    let now = Instant::now();
    let outcome = server.apply_position(
        &position("Me", [0.0, 0.0, 0.0], 90.0, 10.0),
        Some(ClientId(5)),
        now,
    );
    assert_eq!(
        outcome,
        PositionOutcome::Mine {
            speaking: Some(SpeakingState {
                client_id: ClientId(5),
                is_talking_on_radio: false,
            })
        }
    );
    let mine = server.client("Me").unwrap();
    assert_eq!(mine.client_id, ClientId(5));
    assert_eq!(mine.data_frame, 6);
    assert_eq!(mine.position_time, Some(now));
    assert_eq!(mine.view_angle, Angle::from_degrees(90.0));
    assert_eq!(mine.terrain_interception, 3);
    assert!(mine.usable_radios.short_range && mine.usable_radios.diver);
    assert!(!mine.usable_radios.long_range);
    assert_eq!(mine.vehicle.vehicle_name, "1:2");
    assert!((mine.voice_volume_multiplier - 0.8).abs() < 1e-12);
    assert_eq!(server.my_position, Position::ORIGIN);
    assert!(server.can_speak);
}

#[test]
fn the_local_players_pos_without_an_entry_still_sets_the_listener() {
    let mut server = ServerRadioData::new("Me".to_owned());
    let outcome = server.apply_position(
        &position("Me", [1.0, 2.0, 3.0], 0.0, 0.0),
        None,
        Instant::now(),
    );
    assert_eq!(outcome, PositionOutcome::Mine { speaking: None });
    assert_eq!(server.my_position.to_array(), [1.0, 2.0, 3.0]);
}

#[test]
fn another_players_pos_keeps_their_view_and_turns_the_listeners() {
    let mut server = server();
    server.client_mut("Alpha").unwrap().view_angle = Angle::from_degrees(33.0);
    let outcome = server.apply_position(
        &position("Alpha", [10.0, 0.0, 0.0], 90.0, 45.0),
        Some(ClientId(1)),
        Instant::now(),
    );
    assert_eq!(
        outcome,
        PositionOutcome::Other {
            speaking: SpeakingState {
                client_id: ClientId(2),
                is_talking_on_radio: false,
            }
        }
    );
    let alpha = server.client("Alpha").unwrap();
    assert_eq!(alpha.position.to_array(), [10.0, 0.0, 0.0]);
    assert_eq!(alpha.view_angle, Angle::from_degrees(33.0));
    assert_eq!(alpha.data_frame, 6);
    assert_eq!(
        server.client("Me").unwrap().view_angle,
        Angle::from_degrees(45.0)
    );
}

#[test]
fn an_unknown_nickname_asks_for_the_client_list_and_changes_no_entry() {
    let mut server = server();
    let update = position("Charlie", [5.0, 0.0, 0.0], 0.0, 0.0);
    assert_eq!(
        server.apply_position(&update, Some(ClientId(1)), Instant::now()),
        PositionOutcome::UnknownNickname
    );
    assert!(server.client("Charlie").is_none());
    server.update_nicknames(&[(ClientId(3), "Charlie".to_owned())]);
    assert!(matches!(
        server.apply_position(&update, Some(ClientId(1)), Instant::now()),
        PositionOutcome::Other { .. }
    ));
}

#[test]
fn a_held_tangent_makes_the_pos_reply_speaking() {
    let mut server = server();
    server.client_mut("Alpha").unwrap().tangent = Some(TangentKind::ShortRange);
    let PositionOutcome::Other { speaking } = server.apply_position(
        &position("Alpha", [1.0, 0.0, 0.0], 0.0, 0.0),
        None,
        Instant::now(),
    ) else {
        panic!("Alpha is known");
    };
    assert!(speaking.is_talking_on_radio);
}

#[test]
fn is_speaking_reads_the_talking_state_of_a_known_nickname() {
    let mut server = server();
    server.client_mut("Alpha").unwrap().is_talking_now = true;
    assert_eq!(
        server.speaking_state_of("Alpha"),
        Some(SpeakingState {
            client_id: ClientId(2),
            is_talking_on_radio: true,
        })
    );
    assert_eq!(server.speaking_state_of("Nobody"), None);
}
