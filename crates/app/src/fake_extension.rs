//! The extension without REAPER and with its real server: `LinkServer` and the timer loop over
//! `FakeReaper`, so the app's real `LinkConnection` can talk to it over a socket (WP 4.12).

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use link::{Endpoint, LinkServer, ServerError};
use reaper_port::FakeReaper;
use shared_kernel::Seconds;
use timer_loop::{QueuedCommands, StatePublisher, TimerLoop};

/// The real server, ticking on its own thread. Each tick moves the fake REAPER's time on by the
/// same span, takes the queued commands and pushes the new state, as the extension does.
pub struct FakeExtension {
    server: Arc<LinkServer>,
    stop: Arc<AtomicBool>,
    ticker: Option<JoinHandle<()>>,
}

impl FakeExtension {
    /// Starts the server over `reaper`, ticking every `tick`.
    pub fn start(reaper: FakeReaper, tick: Duration) -> Result<Self, ServerError> {
        let (sender, commands) = channel();
        let server = Arc::new(LinkServer::start("fake", QueuedCommands::new(sender))?);
        let stop = Arc::new(AtomicBool::new(false));
        let span = Seconds::new(tick.as_secs_f64()).unwrap_or(Seconds::ZERO);
        let ticker = {
            let server = Arc::clone(&server);
            let stop = Arc::clone(&stop);
            thread::Builder::new()
                .name("fake-extension".into())
                .spawn(move || {
                    let mut timer_loop = TimerLoop::new(reaper);
                    let mut publisher = StatePublisher::default();
                    while !stop.load(Ordering::SeqCst) {
                        while let Ok(command) = commands.try_recv() {
                            timer_loop.link_command(command);
                        }
                        timer_loop.port_mut().advance(span);
                        timer_loop.tick();
                        publisher.publish(&mut timer_loop, &server, |_| {});
                        thread::sleep(tick);
                    }
                })?
        };
        Ok(Self {
            server,
            stop,
            ticker: Some(ticker),
        })
    }

    /// What a client needs to connect straight to the server.
    pub fn endpoint(&self) -> &Endpoint {
        self.server.endpoint()
    }

    /// Where the server listens.
    pub fn address(&self) -> SocketAddr {
        self.server.address()
    }

    /// Clients that are connected and past the handshake.
    pub fn client_count(&self) -> usize {
        self.server.client_count()
    }
}

impl Drop for FakeExtension {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(ticker) = self.ticker.take() {
            let _ = ticker.join();
        }
    }
}
