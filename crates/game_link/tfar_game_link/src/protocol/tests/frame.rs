//! Unit tests for `Frame`, `FrameKind` and the `HELLO` check.

use super::*;

const ALL_KINDS: [FrameKind; 6] = [
    FrameKind::Hello,
    FrameKind::SyncRequest,
    FrameKind::SyncResponse,
    FrameKind::Async,
    FrameKind::Focus,
    FrameKind::Status,
];

#[test]
fn the_kind_codes_are_the_wire_table_one_to_six() {
    let codes: Vec<u8> = ALL_KINDS.iter().map(|kind| kind.code()).collect();
    assert_eq!(codes, [1, 2, 3, 4, 5, 6]);
    for kind in ALL_KINDS {
        assert_eq!(FrameKind::from_code(kind.code()), Some(kind));
    }
    assert_eq!(FrameKind::from_code(0), None);
    assert_eq!(FrameKind::from_code(7), None);
}

#[test]
fn the_kind_names_are_the_wire_table_names() {
    let names: Vec<String> = ALL_KINDS.iter().map(ToString::to_string).collect();
    assert_eq!(
        names,
        ["HELLO", "SYNC_REQ", "SYNC_RESP", "ASYNC", "FOCUS", "STATUS"]
    );
}

#[test]
fn an_own_hello_is_accepted_by_its_own_flavour() {
    for flavour in [Flavour::V0_9, Flavour::V1_0] {
        assert_eq!(Hello::new(flavour).mismatch_with(flavour), None);
        assert_eq!(Frame::hello(flavour), Frame::Hello(Hello::new(flavour)));
    }
}

#[test]
fn a_hello_with_another_magic_is_a_magic_mismatch() {
    let hello = Hello {
        magic: *b"TFAX",
        ..Hello::new(Flavour::V1_0)
    };
    assert_eq!(
        hello.mismatch_with(Flavour::V1_0),
        Some(HelloMismatch::Magic { found: *b"TFAX" })
    );
}

#[test]
fn a_hello_with_another_protocol_is_a_protocol_mismatch() {
    let hello = Hello {
        protocol: 2,
        ..Hello::new(Flavour::V1_0)
    };
    assert_eq!(
        hello.mismatch_with(Flavour::V1_0),
        Some(HelloMismatch::Protocol { found: 2 })
    );
}

#[test]
fn a_hello_of_the_other_flavour_is_a_flavour_mismatch() {
    assert_eq!(
        Hello::new(Flavour::V0_9).mismatch_with(Flavour::V1_0),
        Some(HelloMismatch::Flavour {
            found: 9,
            expected: Flavour::V1_0
        })
    );
}

#[test]
fn a_mismatch_names_the_peer_value_and_the_expected_one() {
    let text = HelloMismatch::Flavour {
        found: 9,
        expected: Flavour::V1_0,
    }
    .to_string();
    assert_eq!(text, "flavour 9 is not 10 (1.0)");
    assert_eq!(
        HelloMismatch::Protocol { found: 7 }.to_string(),
        "protocol 7 is not 1"
    );
}
