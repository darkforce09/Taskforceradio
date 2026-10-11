//! Unit tests for `FrameDecoder`.

use super::*;
use crate::protocol::error::Error;
use crate::protocol::flavour::Flavour;
use crate::protocol::frame_codec::encode;
use crate::protocol::protocol_constants::MAX_FRAME_LENGTH;

fn session_frames() -> Vec<Frame> {
    vec![
        Frame::hello(Flavour::V1_0),
        Frame::Status { needcfg: true },
        Frame::Focus { arma_focused: true },
        Frame::SyncRequest {
            id: 1,
            text: b"TS_INFO\tPING".to_vec(),
        },
        Frame::SyncResponse {
            id: 1,
            text: b"PONG".to_vec(),
        },
        Frame::Async {
            text: b"DFRAME".to_vec(),
        },
        Frame::Async {
            text: vec![b'x'; 300],
        },
    ]
}

fn stream_of(frames: &[Frame]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for frame in frames {
        encode(frame, &mut bytes).unwrap();
    }
    bytes
}

fn drain(decoder: &mut FrameDecoder) -> Vec<Frame> {
    let mut frames = Vec::new();
    while let Some(frame) = decoder.next_frame().unwrap() {
        frames.push(frame);
    }
    frames
}

#[test]
fn a_stream_fed_one_byte_at_a_time_yields_every_frame_in_order() {
    let frames = session_frames();
    let mut decoder = FrameDecoder::new();
    let mut collected = Vec::new();
    for byte in stream_of(&frames) {
        decoder.push(&[byte]);
        collected.extend(drain(&mut decoder));
    }
    assert_eq!(collected, frames);
    assert_eq!(decoder.buffered_len(), 0);
}

#[test]
fn a_stream_fed_in_one_chunk_yields_every_frame_in_order() {
    let frames = session_frames();
    let mut decoder = FrameDecoder::new();
    decoder.push(&stream_of(&frames));
    assert_eq!(drain(&mut decoder), frames);
}

#[test]
fn a_stream_fed_in_uneven_chunks_yields_every_frame_in_order() {
    let frames = session_frames();
    let bytes = stream_of(&frames);
    let mut decoder = FrameDecoder::new();
    let mut collected = Vec::new();
    for chunk in bytes.chunks(7) {
        decoder.push(chunk);
        collected.extend(drain(&mut decoder));
    }
    assert_eq!(collected, frames);
}

#[test]
fn an_incomplete_frame_yields_nothing_and_keeps_its_bytes() {
    let bytes = stream_of(&[Frame::Status { needcfg: true }]);
    let mut decoder = FrameDecoder::new();
    decoder.push(&bytes[..5]);
    assert!(decoder.next_frame().unwrap().is_none());
    assert_eq!(decoder.buffered_len(), 5);
    decoder.push(&bytes[5..]);
    assert_eq!(
        decoder.next_frame().unwrap(),
        Some(Frame::Status { needcfg: true })
    );
}

#[test]
fn a_zero_length_prefix_is_an_error() {
    let mut decoder = FrameDecoder::new();
    decoder.push(&[0, 0, 0, 0, 0x06, 0x01]);
    assert!(matches!(decoder.next_frame(), Err(Error::ZeroLength)));
}

#[test]
fn an_over_length_prefix_is_an_error_before_its_body_arrives() {
    let mut decoder = FrameDecoder::new();
    decoder.push(&(MAX_FRAME_LENGTH + 1).to_le_bytes());
    assert!(matches!(
        decoder.next_frame(),
        Err(Error::FrameTooLong { .. })
    ));
}

#[test]
fn an_unknown_kind_is_an_error() {
    let mut decoder = FrameDecoder::new();
    decoder.push(&[2, 0, 0, 0, 0x09, 0x01]);
    assert!(matches!(
        decoder.next_frame(),
        Err(Error::UnknownKind { kind: 0x09 })
    ));
}

#[test]
fn a_bad_payload_is_an_error() {
    let mut decoder = FrameDecoder::new();
    decoder.push(&[2, 0, 0, 0, 0x05, 0x07]);
    assert!(matches!(
        decoder.next_frame(),
        Err(Error::BadPayload { .. })
    ));
}
