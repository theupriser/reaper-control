//! Length-prefixed framing (SPEC §2.2): a 4-byte big-endian length, then that many payload bytes.
//! The decoder never allocates for a length it will not accept and never panics on any input.

mod decoder;
mod error;

pub use decoder::FrameDecoder;
pub use error::FrameError;

/// Largest payload a frame may carry (1 MiB). Anything bigger is a protocol error.
pub const MAX_FRAME_LEN: usize = 1 << 20;

const HEADER_LEN: usize = 4;

/// Prefix `payload` with its length.
pub fn encode(payload: &[u8]) -> Result<Vec<u8>, FrameError> {
    check_len(payload.len())?;
    let len =
        u32::try_from(payload.len()).map_err(|_| FrameError::TooLarge { len: payload.len() })?;
    let mut out = Vec::with_capacity(HEADER_LEN + payload.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

fn check_len(len: usize) -> Result<(), FrameError> {
    match len {
        0 => Err(FrameError::Empty),
        l if l > MAX_FRAME_LEN => Err(FrameError::TooLarge { len: l }),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;
