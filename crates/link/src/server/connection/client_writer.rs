use std::io::{self, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};

use crate::server::hub::Frame;

/// Writes queued frames to one client on its own thread, so nobody else waits for a slow socket.
pub(super) struct ClientWriter(JoinHandle<()>);

impl ClientWriter {
    pub(super) fn spawn(mut stream: TcpStream, inbox: Receiver<Frame>) -> io::Result<Self> {
        let handle = thread::Builder::new()
            .name("link-write".into())
            .spawn(move || {
                while let Ok(frame) = inbox.recv() {
                    if stream.write_all(&frame).is_err() {
                        break;
                    }
                }
                let _ = stream.shutdown(Shutdown::Both);
            })?;
        Ok(Self(handle))
    }

    pub(super) fn join(self) {
        let _ = self.0.join();
    }
}
