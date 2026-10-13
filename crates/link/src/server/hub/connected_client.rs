use std::net::TcpStream;
use std::sync::Arc;
use std::sync::mpsc::SyncSender;

use crate::server::exit_handles::EXIT_HANDLES;

pub(in crate::server) type Frame = Arc<Vec<u8>>;

pub(in crate::server) struct ConnectedClient {
    pub(super) id: u64,
    pub(super) outbox: SyncSender<Frame>,
    pub(super) stream: TcpStream,
    handle: usize,
}

impl ConnectedClient {
    /// Remembers the stream's handle for the exit hook for as long as the client is held.
    pub(super) fn new(id: u64, outbox: SyncSender<Frame>, stream: TcpStream) -> Self {
        let handle = raw_handle(&stream);
        EXIT_HANDLES.add(handle);
        Self {
            id,
            outbox,
            stream,
            handle,
        }
    }
}

impl Drop for ConnectedClient {
    // Runs before `stream` closes, so a remembered handle never outlives its socket.
    fn drop(&mut self) {
        EXIT_HANDLES.remove(self.handle);
    }
}

#[cfg(unix)]
fn raw_handle(stream: &TcpStream) -> usize {
    use std::os::fd::AsRawFd;
    usize::try_from(stream.as_raw_fd()).unwrap_or(0)
}

#[cfg(windows)]
fn raw_handle(stream: &TcpStream) -> usize {
    use std::os::windows::io::AsRawSocket;
    usize::try_from(stream.as_raw_socket()).unwrap_or(0)
}
