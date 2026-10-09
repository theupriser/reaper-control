use std::path::Path;

#[cfg(windows)]
unsafe extern "system" {
    fn DeleteFileW(path: *const u16) -> i32;
}

#[cfg(unix)]
unsafe extern "C" {
    fn unlink(path: *const std::ffi::c_char) -> i32;
}

/// A file to delete in the C runtime's exit hook. It calls the operating system directly: on
/// Windows the hook runs while the DLL unloads, when Rust's thread-locals are already gone and
/// `std::fs` panics.
#[derive(Debug)]
pub(super) struct ExitFile {
    #[cfg(windows)]
    path: Vec<u16>,
    #[cfg(unix)]
    path: std::ffi::CString,
}

impl ExitFile {
    #[cfg(windows)]
    pub(super) fn new(path: &Path) -> Option<Self> {
        use std::os::windows::ffi::OsStrExt;
        let wide = path.as_os_str().encode_wide().chain(Some(0)).collect();
        Some(Self { path: wide })
    }

    #[cfg(unix)]
    pub(super) fn new(path: &Path) -> Option<Self> {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
        Some(Self { path })
    }

    pub(super) fn remove(&self) {
        // SAFETY: `path` is a NUL-terminated string that lives as long as `self`.
        #[cfg(windows)]
        unsafe {
            DeleteFileW(self.path.as_ptr());
        }
        // SAFETY: as above.
        #[cfg(unix)]
        unsafe {
            unlink(self.path.as_ptr());
        }
    }
}
