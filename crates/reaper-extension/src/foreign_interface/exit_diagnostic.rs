//! TEMPORARY: writes what the exit hooks see into `exit.diag` next to the log, with raw calls.

#[cfg(windows)]
use std::sync::OnceLock;

#[cfg(windows)]
static PATH: OnceLock<Vec<u16>> = OnceLock::new();

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateFileW(name: *const u16, access: u32, share: u32, security: *const u8, disposition: u32, flags: u32, template: usize) -> isize;
    fn WriteFile(file: isize, buffer: *const u8, length: u32, written: *mut u32, overlapped: *const u8) -> i32;
    fn CloseHandle(file: isize) -> i32;
}

#[cfg(windows)]
#[link(name = "ws2_32")]
unsafe extern "system" {
    fn WSAGetLastError() -> i32;
}

#[cfg(windows)]
pub(super) fn prepare(directory: &std::path::Path) {
    use std::os::windows::ffi::OsStrExt;
    let wide = directory.join("exit.diag").as_os_str().encode_wide().chain(Some(0)).collect();
    let _ = PATH.set(wide);
}

#[cfg(not(windows))]
pub(super) fn prepare(_: &std::path::Path) {}

/// Appends `text` and the last socket error; ignores every failure.
#[cfg(windows)]
pub(super) fn note(text: &str, result: i64) {
    let Some(path) = PATH.get() else { return };
    // SAFETY: plain operating system calls on a prepared path and a local buffer.
    unsafe {
        let error = WSAGetLastError();
        let file = CreateFileW(path.as_ptr(), 4, 3, std::ptr::null(), 4, 0x80, 0);
        if file == -1 {
            return;
        }
        let mut line = [0_u8; 96];
        let mut length = 0;
        for byte in text.bytes().chain(*b" result ") {
            if let Some(slot) = line.get_mut(length) {
                *slot = byte;
                length += 1;
            }
        }
        let mut number = result;
        let mut digits = [0_u8; 12];
        let mut count = 0;
        let negative = number < 0;
        number = number.abs();
        loop {
            if let Some(slot) = digits.get_mut(count) {
                *slot = b'0' + u8::try_from(number % 10).unwrap_or(0);
            }
            count += 1;
            number /= 10;
            if number == 0 || count >= 11 {
                break;
            }
        }
        if negative && let Some(slot) = line.get_mut(length) {
            *slot = b'-';
            length += 1;
        }
        while count > 0 {
            count -= 1;
            if let (Some(slot), Some(digit)) = (line.get_mut(length), digits.get(count)) {
                *slot = *digit;
                length += 1;
            }
        }
        for byte in *b" wsa " {
            if let Some(slot) = line.get_mut(length) {
                *slot = byte;
                length += 1;
            }
        }
        let mut number = i64::from(error);
        let mut digits = [0_u8; 12];
        let mut count = 0;
        loop {
            if let Some(slot) = digits.get_mut(count) {
                *slot = b'0' + u8::try_from(number % 10).unwrap_or(0);
            }
            count += 1;
            number /= 10;
            if number == 0 || count >= 11 {
                break;
            }
        }
        while count > 0 {
            count -= 1;
            if let (Some(slot), Some(digit)) = (line.get_mut(length), digits.get(count)) {
                *slot = *digit;
                length += 1;
            }
        }
        if let Some(slot) = line.get_mut(length) {
            *slot = b'\n';
            length += 1;
        }
        let mut written = 0_u32;
        WriteFile(file, line.as_ptr(), u32::try_from(length).unwrap_or(0), &raw mut written, std::ptr::null());
        CloseHandle(file);
    }
}

#[cfg(not(windows))]
pub(super) fn note(_: &str, _: i64) {}
