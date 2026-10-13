//! Tells the connected apps that REAPER is quitting, from the C runtime's exit hook.
//!
//! The hook cannot use threads, locks or `std::io`: on Windows it runs while the DLL unloads, when
//! the other threads are gone and Rust's thread-locals are too. So the frame is built at start,
//! the clients' socket handles are kept in atomics by the link server, and the frame goes out with
//! one `send` call per handle straight to the operating system.

use link::EXIT_HANDLES;

#[cfg(windows)]
#[link(name = "ws2_32")]
unsafe extern "system" {
    fn send(socket: usize, buffer: *const u8, length: i32, flags: i32) -> i32;
}

#[cfg(unix)]
unsafe extern "C" {
    fn send(socket: i32, buffer: *const u8, length: usize, flags: i32) -> isize;
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn setsockopt(socket: i32, level: i32, name: i32, value: *const i32, length: u32) -> i32;
}

/// Writes `frame` to every client the server holds right now. Failures are ignored: the app
/// then closes the way it would after a crash, only later.
pub(super) fn tell_clients(frame: &[u8]) {
    for handle in EXIT_HANDLES.snapshot().into_iter().flatten() {
        write_frame(handle, frame);
    }
}

#[cfg(windows)]
pub(super) fn write_frame(handle: usize, frame: &[u8]) {
    let Ok(length) = i32::try_from(frame.len()) else {
        return;
    };
    // SAFETY: `frame` is valid for `length` bytes; a handle that is not a socket only makes
    // `send` return an error.
    unsafe {
        send(handle, frame.as_ptr(), length, 0);
    }
}

#[cfg(unix)]
pub(super) fn write_frame(handle: usize, frame: &[u8]) {
    let Ok(socket) = i32::try_from(handle) else {
        return;
    };
    // macOS has no `MSG_NOSIGNAL`: the option keeps a closed peer from raising SIGPIPE in REAPER.
    #[cfg(target_os = "macos")]
    // SAFETY: the value is a live `i32` for the four bytes announced.
    unsafe {
        const SOL_SOCKET: i32 = 0xffff;
        const SO_NOSIGPIPE: i32 = 0x1022;
        let on: i32 = 1;
        setsockopt(socket, SOL_SOCKET, SO_NOSIGPIPE, &raw const on, 4);
    }
    #[cfg(target_os = "macos")]
    const FLAGS: i32 = 0;
    #[cfg(not(target_os = "macos"))]
    const FLAGS: i32 = 0x4000; // MSG_NOSIGNAL
    // SAFETY: `frame` is valid for its length; a handle that is not a socket only makes `send`
    // return an error.
    unsafe {
        send(socket, frame.as_ptr(), frame.len(), FLAGS);
    }
}

#[cfg(test)]
mod tests;
