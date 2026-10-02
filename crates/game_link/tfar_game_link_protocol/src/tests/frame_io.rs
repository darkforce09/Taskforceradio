//! Unit tests for `read_frame` and `write_frame`.

use std::io::Cursor;

use super::*;
use crate::flavour::Flavour;

#[test]
fn frames_written_with_write_frame_read_back_in_order() {
    let frames = [
        Frame::hello(Flavour::V0_9),
        Frame::SyncRequest {
            id: 7,
            text: b"FREQ\t".to_vec(),
        },
        Frame::Status { needcfg: false },
    ];
    let mut stream = Vec::new();
    for frame in &frames {
        write_frame(&mut stream, frame).unwrap();
    }
    let mut reader = Cursor::new(stream);
    for frame in &frames {
        assert_eq!(&read_frame(&mut reader).unwrap(), frame);
    }
    assert!(matches!(
        read_frame(&mut reader),
        Err(Error::ConnectionClosed)
    ));
}

#[test]
fn a_stream_ending_inside_a_frame_is_an_input_output_error() {
    let mut bytes = Vec::new();
    write_frame(&mut bytes, &Frame::hello(Flavour::V1_0)).unwrap();
    bytes.truncate(6);
    assert!(matches!(
        read_frame(&mut Cursor::new(bytes)),
        Err(Error::Io { source }) if source.kind() == ErrorKind::UnexpectedEof
    ));
}

#[test]
fn read_frame_refuses_an_over_length_prefix_without_reading_the_body() {
    let mut reader = Cursor::new(u32::MAX.to_le_bytes().to_vec());
    assert!(matches!(
        read_frame(&mut reader),
        Err(Error::FrameTooLong { .. })
    ));
}

#[test]
fn read_frame_refuses_a_zero_length_and_an_unknown_kind() {
    assert!(matches!(
        read_frame(&mut Cursor::new(vec![0, 0, 0, 0])),
        Err(Error::ZeroLength)
    ));
    assert!(matches!(
        read_frame(&mut Cursor::new(vec![1, 0, 0, 0, 0x99])),
        Err(Error::UnknownKind { kind: 0x99 })
    ));
}
