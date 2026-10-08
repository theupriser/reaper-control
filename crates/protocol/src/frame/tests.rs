use std::path::PathBuf;

use serde::Deserialize;

use super::*;

#[derive(Deserialize)]
struct Vector {
    name: String,
    hex: String,
    frames: Vec<String>,
    error: Option<String>,
}

fn unhex(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .filter_map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect()
}

fn drain(decoder: &mut FrameDecoder) -> (Vec<String>, Option<FrameError>) {
    let mut frames = Vec::new();
    loop {
        match decoder.next_frame() {
            Ok(Some(f)) => frames.push(String::from_utf8_lossy(&f).into_owned()),
            Ok(None) => return (frames, None),
            Err(e) => return (frames, Some(e)),
        }
    }
}

fn error_name(e: Option<FrameError>) -> Option<&'static str> {
    e.map(|e| match e {
        FrameError::TooLarge { .. } => "TooLarge",
        FrameError::Empty => "Empty",
    })
}

#[test]
fn vectors_decode_the_same_whole_and_byte_by_byte() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testing/vectors/frames.json");
    let vectors: Vec<Vector> = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    assert!(!vectors.is_empty());
    for v in vectors {
        let bytes = unhex(&v.hex);

        let mut whole = FrameDecoder::new();
        whole.push(&bytes);
        let (frames, error) = drain(&mut whole);
        assert_eq!(frames, v.frames, "{}: frames", v.name);
        assert_eq!(error_name(error), v.error.as_deref(), "{}: error", v.name);

        let mut trickle = FrameDecoder::new();
        let mut got = Vec::new();
        let mut got_err = None;
        for b in &bytes {
            trickle.push(std::slice::from_ref(b));
            let (f, e) = drain(&mut trickle);
            got.extend(f);
            got_err = got_err.or(e);
        }
        assert_eq!(got, v.frames, "{}: byte-by-byte frames", v.name);
        assert_eq!(
            error_name(got_err),
            v.error.as_deref(),
            "{}: byte-by-byte error",
            v.name
        );
    }
    Ok(())
}

#[test]
fn encode_then_decode_returns_the_payload() -> Result<(), FrameError> {
    let mut decoder = FrameDecoder::new();
    decoder.push(&encode(b"abc")?);
    assert_eq!(decoder.next_frame()?, Some(b"abc".to_vec()));
    assert_eq!(decoder.next_frame()?, None);
    Ok(())
}

#[test]
fn encode_refuses_empty_and_oversized_payloads() {
    assert_eq!(encode(b""), Err(FrameError::Empty));
    let big = vec![0u8; MAX_FRAME_LENGTH + 1];
    assert_eq!(
        encode(&big),
        Err(FrameError::TooLarge {
            length: MAX_FRAME_LENGTH + 1
        })
    );
    assert!(encode(&big[..MAX_FRAME_LENGTH]).is_ok());
}

#[test]
fn a_failed_decoder_stays_failed() {
    let mut decoder = FrameDecoder::new();
    decoder.push(&[0xff, 0xff, 0xff, 0xff]);
    let first = decoder.next_frame();
    assert!(matches!(first, Err(FrameError::TooLarge { .. })));
    decoder.push(&[0, 0, 0, 1, b'x']);
    assert_eq!(decoder.next_frame(), first);
}

#[test]
fn an_oversized_length_is_refused_before_any_payload_arrives() {
    let mut decoder = FrameDecoder::new();
    decoder.push(
        &u32::try_from(MAX_FRAME_LENGTH + 1)
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    assert!(decoder.next_frame().is_err());
}
