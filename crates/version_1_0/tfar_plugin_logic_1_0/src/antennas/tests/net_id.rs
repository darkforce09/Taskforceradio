//! Unit tests for `NetId`.

use super::*;

#[test]
fn an_arma_net_id_reads_creator_and_object() {
    let id = NetId::parse("2:390");
    assert_eq!(id, NetId::new(2, 390));
    assert_eq!((id.creator(), id.object()), (2, 390));
    assert!(!id.is_null());
}

#[test]
fn a_net_id_without_a_colon_is_null() {
    assert_eq!(NetId::parse("2390"), NetId::NULL);
    assert_eq!(NetId::parse(""), NetId::NULL);
    assert!(NetId::NULL.is_null());
}

/// The original runs `strchr` over a `string_view` that is not terminated, so an id without a
/// colon borrows the colon of the text after it; here the colon is searched within the id only.
#[test]
fn net_id_parse_never_reads_past_its_own_text() {
    let message = "2390;1000;[1:2,3]";
    let id_text = message.split(';').next().unwrap();
    assert_eq!(NetId::parse(id_text), NetId::NULL);
}

#[test]
fn a_malformed_side_is_the_null_id() {
    assert_eq!(NetId::parse("x:5"), NetId::NULL);
    assert_eq!(NetId::parse("2:"), NetId::NULL);
}

#[test]
fn object_zero_is_the_null_id() {
    assert!(NetId::parse("2:0").is_null());
}

#[test]
fn a_net_id_displays_as_arma_writes_it() {
    assert_eq!(NetId::new(2, 390).to_string(), "2:390");
}
