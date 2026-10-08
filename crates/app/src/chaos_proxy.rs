//! A loopback proxy between the app and the extension's server that breaks the link on demand:
//! delayed, reordered and cut frames, and bytes that are not frames at all (WP 4.12).

mod frame_relay;
mod relayed_connection;

use std::io;
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::chaos_settings::ChaosSettings;
use relayed_connection::RelayedConnection;

/// How often the accept loop looks for a stop request.
const ACCEPT_POLL: Duration = Duration::from_millis(10);

/// Accepts the app's connections and relays each to the server, applying the current settings.
/// Dropping it closes everything.
pub struct ChaosProxy {
    address: SocketAddr,
    settings: Arc<Mutex<ChaosSettings>>,
    stop: Arc<AtomicBool>,
    accept: Option<JoinHandle<()>>,
}

impl ChaosProxy {
    /// Listens on a free loopback port and relays to `upstream`.
    pub fn start(upstream: SocketAddr) -> io::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let settings = Arc::new(Mutex::new(ChaosSettings::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let accept = {
            let settings = Arc::clone(&settings);
            let stop = Arc::clone(&stop);
            thread::Builder::new()
                .name("chaos-accept".into())
                .spawn(move || accept_loop(&listener, upstream, &settings, &stop))?
        };
        Ok(Self {
            address,
            settings,
            stop,
            accept: Some(accept),
        })
    }

    /// Where the app should connect instead of the server.
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Changes the faults; running connections pick them up with their next frame.
    pub fn set(&self, settings: ChaosSettings) {
        if let Ok(mut current) = self.settings.lock() {
            *current = settings;
        }
    }
}

fn accept_loop(
    listener: &TcpListener,
    upstream: SocketAddr,
    settings: &Arc<Mutex<ChaosSettings>>,
    stop: &Arc<AtomicBool>,
) {
    while !stop.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((client, _)) => {
                let Ok(server) = TcpStream::connect(upstream) else {
                    continue;
                };
                RelayedConnection::start(client, server, settings, stop);
            }
            Err(_) => thread::sleep(ACCEPT_POLL),
        }
    }
}

impl Drop for ChaosProxy {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(accept) = self.accept.take() {
            let _ = accept.join();
        }
    }
}
