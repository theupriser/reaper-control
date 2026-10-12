//! Which processor a program file was built for, read from its header.

use std::fs::File;
use std::io::Read;
use std::path::Path;

const HEADER_BYTES: u64 = 4096;
const MACH_O_64_LITTLE: [u8; 4] = [0xcf, 0xfa, 0xed, 0xfe];
const MACH_O_UNIVERSAL: [u8; 4] = [0xca, 0xfe, 0xba, 0xbe];
const MACH_O_ARM64: u32 = 0x0100_000c;
const MACH_O_X86_64: u32 = 0x0100_0007;
const PORTABLE_EXECUTABLE_X64: u16 = 0x8664;
const PORTABLE_EXECUTABLE_ARM64: u16 = 0xaa64;

/// The processor a REAPER (or extension) file is built for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryArchitecture {
    /// Apple Silicon or Windows on Arm.
    Arm64,
    /// Intel or AMD 64-bit.
    X64,
    /// Recognised as a program, but not one this app supports (for example 32-bit).
    Other,
}

impl BinaryArchitecture {
    /// Reads the header of the file at `path`; `None` when it is not a program file.
    pub fn read(path: &Path) -> std::io::Result<Option<Self>> {
        let mut header = Vec::new();
        File::open(path)?
            .take(HEADER_BYTES)
            .read_to_end(&mut header)?;
        Ok(Self::from_header(&header))
    }

    /// Reads the architecture from the first bytes of a Mach-O or Windows program file. A
    /// universal Mach-O file counts as Arm64 when it holds an Arm64 part.
    #[must_use]
    pub fn from_header(header: &[u8]) -> Option<Self> {
        let magic = header.get(..4)?;
        if magic == MACH_O_64_LITTLE {
            return Some(Self::from_mach_o_cpu(read_u32(header, 4, false)?));
        }
        if magic == MACH_O_UNIVERSAL {
            return universal_architecture(header);
        }
        if magic.get(..2)? == b"MZ" {
            return windows_architecture(header);
        }
        None
    }

    fn from_mach_o_cpu(cpu: u32) -> Self {
        match cpu {
            MACH_O_ARM64 => Self::Arm64,
            MACH_O_X86_64 => Self::X64,
            _ => Self::Other,
        }
    }
}

fn universal_architecture(header: &[u8]) -> Option<BinaryArchitecture> {
    let count = read_u32(header, 4, true)?.min(8);
    let cpus: Vec<u32> = (0..count as usize)
        .filter_map(|index| read_u32(header, 8 + index * 20, true))
        .collect();
    if cpus.contains(&MACH_O_ARM64) {
        Some(BinaryArchitecture::Arm64)
    } else if cpus.contains(&MACH_O_X86_64) {
        Some(BinaryArchitecture::X64)
    } else {
        Some(BinaryArchitecture::Other)
    }
}

fn windows_architecture(header: &[u8]) -> Option<BinaryArchitecture> {
    let signature_at = read_u32(header, 0x3c, false)? as usize;
    if header.get(signature_at..signature_at + 4)? != b"PE\0\0" {
        return None;
    }
    let machine = u16::from_le_bytes(
        header
            .get(signature_at + 4..signature_at + 6)?
            .try_into()
            .ok()?,
    );
    Some(match machine {
        PORTABLE_EXECUTABLE_X64 => BinaryArchitecture::X64,
        PORTABLE_EXECUTABLE_ARM64 => BinaryArchitecture::Arm64,
        _ => BinaryArchitecture::Other,
    })
}

fn read_u32(bytes: &[u8], at: usize, big_endian: bool) -> Option<u32> {
    let four: [u8; 4] = bytes.get(at..at + 4)?.try_into().ok()?;
    Some(if big_endian {
        u32::from_be_bytes(four)
    } else {
        u32::from_le_bytes(four)
    })
}
