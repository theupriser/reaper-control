use std::io;
use std::net::{TcpListener, TcpStream};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use super::connection::Connection;
use super::shared::Shared;

const MAX_CONNECTIONS: usize = 16;
const POLL: Duration = Duration::from_millis(20);

/// Takes new sockets from the listener and gives each its own thread, up to a cap.
pub(super) struct Acceptor {
    pub(super) listener: TcpListener,
    pub(super) shared: Arc<Shared>,
}

impl Acceptor {
    pub(super) fn run(self) {
        while !self.shared.stop.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, _)) => self.admit(stream),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => thread::sleep(POLL),
                Err(_) => thread::sleep(POLL),
            }
        }
    }

    fn admit(&self, stream: TcpStream) {
        if self.shared.connections.fetch_add(1, Ordering::SeqCst) >= MAX_CONNECTIONS {
            self.shared.connections.fetch_sub(1, Ordering::SeqCst);
            return;
        }
        let shared = Arc::clone(&self.shared);
        let spawned = thread::Builder::new()
            .name("link-connection".into())
            .spawn(move || {
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    let _ = Connection {
                        stream,
                        shared: &shared,
                    }
                    .run();
                }));
                shared.connections.fetch_sub(1, Ordering::SeqCst);
            });
        drop(spawned);
    }
}
