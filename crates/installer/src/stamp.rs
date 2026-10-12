//! A note left next to an installed extension saying which bundled file it came from.
//! Preparing the copy (signing on macOS) changes its bytes, so the copy can no longer be compared
//! with the bundled file directly.

use std::fs;
use std::path::{Path, PathBuf};

/// Where the stamp for `library` is kept.
#[must_use]
pub fn stamp_path(library: &Path) -> PathBuf {
    let mut name = library.as_os_str().to_os_string();
    name.push(".stamp");
    PathBuf::from(name)
}

/// Records that `library` was made from `bundled`. Failing to write it only means the next look
/// offers the install again.
pub fn write_stamp(bundled: &Path, library: &Path) {
    if let Some(text) = describe(bundled, library) {
        let _ = fs::write(stamp_path(library), text);
    }
}

/// Whether `library` is still the file that was made from `bundled`.
#[must_use]
pub fn stamp_matches(bundled: &Path, library: &Path) -> bool {
    match (
        describe(bundled, library),
        fs::read_to_string(stamp_path(library)),
    ) {
        (Some(expected), Ok(found)) => expected == found,
        _ => false,
    }
}

fn describe(bundled: &Path, library: &Path) -> Option<String> {
    let bytes = fs::read(bundled).ok()?;
    let length = fs::metadata(library).ok()?.len();
    Some(format!("{:016x} {length}", fingerprint(&bytes)))
}

/// FNV-1a: small, stable across Rust versions, and enough to tell two builds apart.
fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
    })
}
