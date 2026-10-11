//! Unit tests for `encode`, `frame_body_length` and `decode_frame_body`.

use super::*;
use crate::protocol::flavour::Flavour;

fn one_frame_of_every_kind() -> Vec<Frame> {
    vec![
        Frame::hello(Flavour::V0_9),
        Frame::hello(Flavour::V1_0),
        Frame::SyncRequest {
            id: 1,
            text: b"TS_INFO\tPING".to_vec(),
        },
        Frame::SyncRequest {
            id: u32::MAX,
            text: Vec::new(),
        },
        Frame::SyncResponse {
            id: 300,
            text: b"PONG".to_vec(),
        },
        Frame::Async {
            text: b"SETCFG\tfull_duplex\ttrue\tBOOL".to_vec(),
        },
        Frame::Async {
            text: vec![0xFF, 0x00, 0xC3, 0xA9],
        },
        Frame::Focus { arma_focused: true },
        Frame::Focus {
            arma_focused: false,
        },
        Frame::Status { needcfg: true },
        Frame::Status { needcfg: false },
    ]
}

fn encoded(frame: &Frame) -> Vec<u8> {
    let mut bytes = Vec::new();
    encode(frame, &mut bytes).unwrap();
    bytes
}

#[test]
fn every_kind_round_trips_through_encode_and_decode() {
    for frame in one_frame_of_every_kind() {
        let bytes = encoded(&frame);
        let prefix: [u8; 4] = bytes[..4].try_into().unwrap();
        assert_eq!(frame_body_length(prefix).unwrap(), bytes.len() - 4);
        assert_eq!(decode_frame_body(&bytes[4..]).unwrap(), frame);
        assert_eq!(bytes[4], frame.kind().code());
    }
}

#[test]
fn encode_appends_after_what_the_buffer_already_holds() {
    let mut bytes = b"xyz".to_vec();
    encode(&Frame::Status { needcfg: true }, &mut bytes).unwrap();
    assert_eq!(bytes, [b'x', b'y', b'z', 2, 0, 0, 0, 0x06, 0x01]);
}

#[test]
fn the_hello_golden_bytes_are_length_kind_magic_protocol_flavour() {
    assert_eq!(
        encoded(&Frame::hello(Flavour::V1_0)),
        [
            0x07, 0x00, 0x00, 0x00, 0x01, b'T', b'F', b'A', b'R', 0x01, 0x0A
        ]
    );
    assert_eq!(
        encoded(&Frame::hello(Flavour::V0_9)),
        [
            0x07, 0x00, 0x00, 0x00, 0x01, b'T', b'F', b'A', b'R', 0x01, 0x09
        ]
    );
}

#[test]
fn the_sync_request_golden_bytes_are_length_kind_id_text_length_text() {
    let mut expected = vec![0x0F, 0x00, 0x00, 0x00, 0x02, 0x01, 0x0C];
    expected.extend_from_slice(b"TS_INFO\tPING");
    let frame = Frame::SyncRequest {
        id: 1,
        text: b"TS_INFO\tPING".to_vec(),
    };
    assert_eq!(encoded(&frame), expected);
    assert_eq!(decode_frame_body(&expected[4..]).unwrap(), frame);
}

#[test]
fn the_small_frames_have_their_golden_bytes() {
    assert_eq!(
        encoded(&Frame::Focus { arma_focused: true }),
        [2, 0, 0, 0, 0x05, 0x01]
    );
    assert_eq!(
        encoded(&Frame::Status { needcfg: false }),
        [2, 0, 0, 0, 0x06, 0x00]
    );
    assert_eq!(
        encoded(&Frame::Async {
            text: b"DFRAME".to_vec()
        }),
        [8, 0, 0, 0, 0x04, 0x06, b'D', b'F', b'R', b'A', b'M', b'E']
    );
    assert_eq!(
        encoded(&Frame::SyncResponse {
            id: 128,
            text: b"OK".to_vec()
        }),
        [6, 0, 0, 0, 0x03, 0x80, 0x01, 0x02, b'O', b'K']
    );
}

#[test]
fn a_zero_length_prefix_is_refused() {
    assert!(matches!(frame_body_length([0; 4]), Err(Error::ZeroLength)));
    assert!(matches!(decode_frame_body(&[]), Err(Error::ZeroLength)));
}

#[test]
fn a_length_above_one_mebibyte_is_refused_and_the_limit_itself_is_legal() {
    assert_eq!(
        frame_body_length(MAX_FRAME_LENGTH.to_le_bytes()).unwrap(),
        1_048_576
    );
    let over = (MAX_FRAME_LENGTH + 1).to_le_bytes();
    assert!(matches!(
        frame_body_length(over),
        Err(Error::FrameTooLong {
            length: 1_048_577,
            limit: 1_048_576
        })
    ));
    assert!(matches!(
        frame_body_length(u32::MAX.to_le_bytes()),
        Err(Error::FrameTooLong { .. })
    ));
}

#[test]
fn encoding_a_text_past_the_limit_is_refused_and_leaves_the_buffer_unchanged() {
    let mut bytes = b"kept".to_vec();
    let frame = Frame::Async {
        text: vec![b'a'; 1_048_576],
    };
    assert!(matches!(
        encode(&frame, &mut bytes),
        Err(Error::FrameTooLong { .. })
    ));
    assert_eq!(bytes, b"kept");
}

#[test]
fn the_longest_legal_text_encodes_and_decodes() {
    // Body: kind (1) + varint length of the text (3 bytes for 1_048_572) + the text.
    let frame = Frame::Async {
        text: vec![b'a'; 1_048_572],
    };
    let bytes = encoded(&frame);
    assert_eq!(bytes.len() - 4, 1_048_576);
    assert_eq!(decode_frame_body(&bytes[4..]).unwrap(), frame);
}

#[test]
fn an_unknown_kind_byte_is_refused() {
    for kind in [0x00, 0x07, 0xFF] {
        assert!(matches!(
            decode_frame_body(&[kind, 0x01]),
            Err(Error::UnknownKind { kind: found }) if found == kind
        ));
    }
}

#[test]
fn a_payload_that_does_not_decode_is_a_bad_payload() {
    // A bool of 2, a short HELLO, a text length past the end, an empty sync request.
    let cases: [(&[u8], FrameKind); 4] = [
        (&[0x06, 0x02], FrameKind::Status),
        (&[0x01, b'T', b'F', b'A', b'R', 0x01], FrameKind::Hello),
        (&[0x04, 0x05, b'a'], FrameKind::Async),
        (&[0x02], FrameKind::SyncRequest),
    ];
    for (body, expected_kind) in cases {
        assert!(
            matches!(
                decode_frame_body(body),
                Err(Error::BadPayload { kind, .. }) if kind == expected_kind
            ),
            "{body:02x?}"
        );
    }
}

#[test]
fn a_payload_with_bytes_left_over_is_refused() {
    assert!(matches!(
        decode_frame_body(&[0x05, 0x01, 0x00, 0x00]),
        Err(Error::TrailingPayloadBytes {
            kind: FrameKind::Focus,
            count: 2
        })
    ));
}

#[test]
fn an_error_names_the_frame_kind_it_belongs_to() {
    let error = decode_frame_body(&[0x06, 0x02]).unwrap_err();
    assert!(error.to_string().starts_with("the STATUS frame payload"));
    let error = decode_frame_body(&[0x2A]).unwrap_err();
    assert_eq!(
        error.to_string(),
        "a game link frame has the unknown kind byte 0x2a"
    );
}
