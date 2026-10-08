//! Feeds arbitrary bytes to the frame decoder in arbitrary chunk sizes.
//! Invariants: no panic, no frame above the limit, same result however the bytes are split.
#![no_main]

use libfuzzer_sys::fuzz_target;
use protocol::frame::{FrameDecoder, MAX_FRAME_LENGTH};

fn run(data: &[u8], chunk: usize) -> (Vec<Vec<u8>>, bool) {
    let mut decoder = FrameDecoder::new();
    let mut frames = Vec::new();
    for piece in data.chunks(chunk) {
        decoder.push(piece);
        loop {
            match decoder.next_frame() {
                Ok(Some(f)) => {
                    assert!(!f.is_empty() && f.len() <= MAX_FRAME_LENGTH);
                    frames.push(f);
                }
                Ok(None) => break,
                Err(_) => return (frames, true),
            }
        }
    }
    (frames, false)
}

fuzz_target!(|data: &[u8]| {
    let (chunk, rest) = match data.split_first() {
        Some((c, rest)) => (usize::from(*c).max(1), rest),
        None => return,
    };
    assert_eq!(run(rest, chunk), run(rest, 1));
    assert_eq!(run(rest, chunk), run(rest, rest.len().max(1)));
});
