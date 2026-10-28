use std::io::Cursor;

use rikka_gecko_protocol::frame::{
    Decoder, FrameError, MAX_FRAME, encode, read_frame, write_frame,
};
use rikka_gecko_protocol::message::{Closed, Hello, Release};
use rikka_gecko_protocol::{CallbackId, EmbedderMessage, GeckoMessage, PROTOCOL, WindowId};
use serde_json::Value;

fn hello() -> GeckoMessage {
    GeckoMessage::Hello(Hello {
        protocol: PROTOCOL,
        version: "158.0.1".to_owned(),
    })
}

fn closed() -> GeckoMessage {
    GeckoMessage::Closed(Closed {
        window: WindowId::new("a1").unwrap(),
    })
}

fn framed(json: &[u8]) -> Vec<u8> {
    let mut bytes = (json.len() as u32).to_le_bytes().to_vec();
    bytes.extend_from_slice(json);
    bytes
}

#[test]
fn round_trips_through_a_stream() {
    let mut stream = Vec::new();
    write_frame(&mut stream, &hello()).unwrap();
    write_frame(&mut stream, &closed()).unwrap();

    let mut reader = Cursor::new(stream);
    assert_eq!(
        read_frame::<_, GeckoMessage>(&mut reader).unwrap(),
        Some(hello())
    );
    assert_eq!(
        read_frame::<_, GeckoMessage>(&mut reader).unwrap(),
        Some(closed())
    );
    assert_eq!(read_frame::<_, GeckoMessage>(&mut reader).unwrap(), None);
}

#[test]
fn encodes_a_little_endian_length_prefix() {
    let release = EmbedderMessage::Release(Release {
        callback: CallbackId::FIRST,
    });
    let frame = encode(&release).unwrap();
    let payload = br#"{"type":"release","callback":1}"#;
    assert_eq!(frame, framed(payload));
}

#[test]
fn decodes_incrementally_byte_by_byte() {
    let mut bytes = encode(&hello()).unwrap();
    bytes.extend(encode(&closed()).unwrap());

    let mut decoder = Decoder::new();
    let mut decoded = Vec::new();
    for byte in bytes {
        decoder.push(&[byte]);
        while let Some(message) = decoder.decode::<GeckoMessage>().unwrap() {
            decoded.push(message);
        }
    }
    assert_eq!(decoded, vec![hello(), closed()]);
    assert_eq!(decoder.buffered(), 0);
    decoder.finish().unwrap();
}

#[test]
fn rejects_frames_over_the_limit() {
    let oversized = (MAX_FRAME as u32 + 1).to_le_bytes();

    let mut reader = Cursor::new(oversized.to_vec());
    let error = read_frame::<_, Value>(&mut reader).unwrap_err();
    assert!(matches!(error, FrameError::TooLarge { length } if length == MAX_FRAME + 1));

    let mut decoder = Decoder::new();
    decoder.push(&oversized);
    let error = decoder.decode::<Value>().unwrap_err();
    assert!(matches!(error, FrameError::TooLarge { .. }));

    let huge = Value::String("x".repeat(MAX_FRAME));
    assert!(
        matches!(encode(&huge), Err(FrameError::TooLarge { length }) if length == MAX_FRAME + 2)
    );
}

#[test]
fn accepts_a_frame_at_the_limit() {
    let at_limit = Value::String("x".repeat(MAX_FRAME - 2));
    let frame = encode(&at_limit).unwrap();
    assert_eq!(frame.len(), MAX_FRAME + 4);
    let mut reader = Cursor::new(frame);
    assert_eq!(read_frame::<_, Value>(&mut reader).unwrap(), Some(at_limit));
}

#[test]
fn reports_truncated_input() {
    let frame = encode(&hello()).unwrap();
    let cases = [(&frame[..2], 2), (&frame[..frame.len() - 5], 5)];
    for (partial, missing) in cases {
        let mut reader = Cursor::new(partial.to_vec());
        let error = read_frame::<_, GeckoMessage>(&mut reader).unwrap_err();
        assert!(
            matches!(error, FrameError::Truncated { missing: reported } if reported == missing),
            "{error:?}"
        );

        let mut decoder = Decoder::new();
        decoder.push(partial);
        assert_eq!(decoder.decode::<GeckoMessage>().unwrap(), None);
        let error = decoder.finish().unwrap_err();
        assert!(
            matches!(error, FrameError::Truncated { missing: reported } if reported == missing),
            "{error:?}"
        );
    }
}

#[test]
fn skips_a_malformed_frame_and_keeps_decoding() {
    let mut decoder = Decoder::new();
    decoder.push(&framed(b"not json"));
    decoder.push(&encode(&closed()).unwrap());
    assert!(matches!(
        decoder.decode::<GeckoMessage>(),
        Err(FrameError::Json(_))
    ));
    assert_eq!(decoder.decode::<GeckoMessage>().unwrap(), Some(closed()));
}

#[test]
fn replaces_unpaired_surrogate_escapes_from_javascript_json() {
    let cases: [(&[u8], &str); 5] = [
        (br#""\ud83d\ude00""#, "\u{1F600}"),
        (br#""a\ud83db""#, "a\u{FFFD}b"),
        (br#""\ude00\ud83d""#, "\u{FFFD}\u{FFFD}"),
        (br#""\\ud83d""#, "\\ud83d"),
        (br#""\ud83d\n""#, "\u{FFFD}\n"),
    ];
    for (json, expected) in cases {
        let mut decoder = Decoder::new();
        decoder.push(&framed(json));
        assert_eq!(
            decoder.decode::<String>().unwrap().as_deref(),
            Some(expected)
        );
        let mut reader = Cursor::new(framed(json));
        assert_eq!(
            read_frame::<_, String>(&mut reader).unwrap().as_deref(),
            Some(expected)
        );
    }
}
