//! Unit tests for `RadioSubtype`.

use super::*;

const KNOWN: [(&str, RadioSubtype, Option<&str>); 5] = [
    ("digital", RadioSubtype::Digital, Some("sw")),
    ("digital_lr", RadioSubtype::DigitalLongRange, Some("lr")),
    ("dd", RadioSubtype::DiverDigital, Some("dd")),
    ("airborne", RadioSubtype::Airborne, Some("ab")),
    ("phone", RadioSubtype::Phone, None),
];

#[test]
fn every_known_name_parses_to_its_subtype() {
    for (name, subtype, _) in KNOWN {
        assert_eq!(RadioSubtype::parse(name), subtype, "name {name:?}");
    }
}

#[test]
fn as_str_gives_back_the_wire_name() {
    for (name, subtype, _) in KNOWN {
        assert_eq!(subtype.as_str(), name);
    }
    assert_eq!(RadioSubtype::parse("analog").as_str(), "analog");
}

#[test]
fn a_name_of_a_known_length_is_not_a_known_subtype() {
    // Same lengths as dd, phone, digital, airborne and digital_lr.
    for name in ["sw", "radio", "analogs", "longwave", "abcdefghij"] {
        assert_eq!(
            RadioSubtype::parse(name),
            RadioSubtype::Unknown(name.to_owned()),
            "name {name:?}"
        );
    }
}

#[test]
fn matching_is_exact() {
    for name in ["Digital", "digital ", " dd", "PHONE", ""] {
        assert!(matches!(
            RadioSubtype::parse(name),
            RadioSubtype::Unknown(_)
        ));
    }
}

#[test]
fn sound_folders_follow_the_subtype() {
    for (_, subtype, folder) in KNOWN {
        assert_eq!(subtype.sound_folder(), folder);
    }
    assert_eq!(RadioSubtype::parse("analog").sound_folder(), None);
}
