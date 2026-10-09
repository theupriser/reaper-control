//! The two systems the extension is built for, and where REAPER keeps things on each.

use std::path::{Path, PathBuf};

use crate::binary_architecture::BinaryArchitecture;

/// macOS on Apple Silicon or Windows on x64 (decision D8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallPlatform {
    /// macOS on Apple Silicon.
    MacArm64,
    /// Windows on x64.
    WindowsX64,
}

impl InstallPlatform {
    /// The platform this app was built for; `None` on systems the app does not support.
    #[must_use]
    pub fn current() -> Option<Self> {
        if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            Some(Self::MacArm64)
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            Some(Self::WindowsX64)
        } else {
            None
        }
    }

    /// The name the extension has inside `UserPlugins`.
    #[must_use]
    pub fn library_file_name(self) -> &'static str {
        match self {
            Self::MacArm64 => "reaper_rc2-arm64.dylib",
            Self::WindowsX64 => "reaper_rc2-x64.dll",
        }
    }

    /// The processor REAPER must be built for to load the extension.
    #[must_use]
    pub fn expected_architecture(self) -> BinaryArchitecture {
        match self {
            Self::MacArm64 => BinaryArchitecture::Arm64,
            Self::WindowsX64 => BinaryArchitecture::X64,
        }
    }

    /// REAPER's resource folder for a normal install. `user_folder` is the home folder on macOS
    /// and the application data folder on Windows.
    #[must_use]
    pub fn default_resource_folder(self, user_folder: &Path) -> PathBuf {
        match self {
            Self::MacArm64 => user_folder.join("Library/Application Support/REAPER"),
            Self::WindowsX64 => user_folder.join("REAPER"),
        }
    }

    /// Where a normal install keeps the REAPER program.
    #[must_use]
    pub fn default_executable(self) -> PathBuf {
        match self {
            Self::MacArm64 => PathBuf::from("/Applications/REAPER.app/Contents/MacOS/REAPER"),
            Self::WindowsX64 => PathBuf::from(r"C:\Program Files\REAPER (x64)\reaper.exe"),
        }
    }

    /// Where a program sits inside a REAPER folder the user chose (a portable install).
    #[must_use]
    pub fn executable_in(self, folder: &Path) -> PathBuf {
        match self {
            Self::MacArm64 => folder.join("REAPER.app/Contents/MacOS/REAPER"),
            Self::WindowsX64 => folder.join("reaper.exe"),
        }
    }
}
