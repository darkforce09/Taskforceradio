//! Unit tests for the local push-to-talk, against the tangent branch in
//! `reference/tfar_0_9_12/ts/src/plugin.cpp:1812-1834`.

use super::*;
use crate::client_state::client_id::ClientId;

#[test]
fn a_press_and_its_release_are_changes_and_a_repeat_is_not() {
    let mut server = ServerRadioData::new("Me".to_owned());
    assert!(server.apply_local_tangent(TangentKind::ShortRange, true, &RadioSubtype::Digital));
    assert!(server.is_tangent_pressed);
    assert!(!server.apply_local_tangent(TangentKind::ShortRange, true, &RadioSubtype::Digital));
    assert!(server.apply_local_tangent(TangentKind::ShortRange, false, &RadioSubtype::Digital));
    assert!(!server.is_tangent_pressed);
}

#[test]
fn the_local_entry_may_use_the_keyed_family_with_its_subtype() {
    let mut server = ServerRadioData::new("Me".to_owned());
    server.update_nicknames(&[(ClientId(1), "Me".to_owned())]);
    server.apply_local_tangent(TangentKind::LongRange, true, &RadioSubtype::Airborne);
    let mine = server.my_client().unwrap();
    assert!(mine.can_use(TangentKind::LongRange));
    assert!(!mine.can_use(TangentKind::ShortRange));
    assert_eq!(mine.subtype, RadioSubtype::Airborne);
    assert_eq!(
        mine.tangent, None,
        "the local entry's held family is set by the plugin command"
    );
}
