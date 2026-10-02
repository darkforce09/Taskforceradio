//! Unit tests for the `TRACK` parser, against `parseCustomVariables`
//! (`reference/tfar_1_0/ts/src/task_force_radio.cpp:235-283`) and `fnc_sessionTracker.sqf:22-46`.

use super::TrackerCustomVariable;
use crate::{GameCommand, TrackingEvent, parse};

fn event(text: &str) -> TrackingEvent {
    match parse(text).unwrap() {
        GameCommand::Track(event) => event,
        other => panic!("not TRACK: {other:?}"),
    }
}

fn variable(id: u32, name: &str, value: &str) -> TrackerCustomVariable {
    TrackerCustomVariable {
        id,
        name: name.to_owned(),
        value: value.to_owned(),
    }
}

#[test]
fn a_tracking_event_reads_action_uid_and_variables() {
    let event = event(
        "TRACK\tstart\t76561198000000000\t[[1,\"group\",4],[5,\"faction\",\"BLU_F\"],[20,\"playerSide\",WEST]]",
    );
    assert_eq!(event.action, "start");
    assert_eq!(event.uid, "76561198000000000");
    assert_eq!(
        event.custom_variables,
        [
            variable(1, "group", "4"),
            variable(5, "faction", "BLU_F"),
            variable(20, "playerSide", "WEST"),
        ]
    );
}

#[test]
fn whitespace_is_dropped_everywhere_as_operator_extraction_drops_it() {
    let event = event("TRACK\tbeta\tuid\t [ [ 8 , \"mission Name\" , \"Op Two\" ] ]");
    assert_eq!(
        event.custom_variables,
        [variable(8, "missionName", "OpTwo")]
    );
}

#[test]
fn a_value_runs_to_its_closing_bracket_commas_included() {
    let event = event("TRACK\tstart\tuid\t[[3,\"name\",\"a,b\"]]");
    assert_eq!(event.custom_variables, [variable(3, "name", "a,b")]);
}

#[test]
fn an_empty_array_has_no_variables() {
    assert_eq!(event("TRACK\tstart\tuid\t[]").custom_variables.len(), 0);
}

#[test]
fn a_malformed_group_or_id_rejects_the_message() {
    assert!(parse("TRACK\tstart\tuid\t[[x,\"name\",1]]").is_err());
    assert!(parse("TRACK\tstart\tuid\t[[1,\"name\"]]").is_err());
    assert!(parse("TRACK\tstart\tuid").is_err());
}
