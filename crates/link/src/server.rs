//! The extension's side: accepts the app, pushes state, answers commands. Every socket
//! operation runs on its own thread, so a slow or broken client never holds up the caller.

mod command_handler;
mod connected_client;
mod connection;
mod hub;
mod server_error;
mod shared;

pub use command_handler::CommandHandler;
pub use server_error::ServerError;

use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use protocol::AppState;
use protocol::message::{ServerMessage, encode_message};

use crate::Endpoint;
use connection::accept_loop;
use hub::Hub;
use shared::Shared;

/// A running server. Dropping it stops it and closes every client.
pub struct LinkServer {
    shared: Arc<Shared>,
    endpoint: Endpoint,
    accept: Option<JoinHandle<()>>,
}

impl LinkServer {
    /// Listen on a free loopback port with a fresh token.
    pub fn start(
        extension_version: &str,
        handler: impl CommandHandler,
    ) -> Result<Self, ServerError> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        listener.set_nonblocking(true)?;
        let endpoint = Endpoint {
            port: listener.local_addr()?.port(),
            token: Endpoint::new_token()?,
        };
        let shared = Arc::new(Shared {
            token: endpoint.token.clone(),
            handler: Box::new(handler),
            extension_version: extension_version.to_owned(),
            hub: Mutex::new(Hub::default()),
            stop: AtomicBool::new(false),
            connections: AtomicUsize::new(0),
        });
        let accept = {
            let shared = Arc::clone(&shared);
            thread::Builder::new()
                .name("link-accept".into())
                .spawn(move || accept_loop(&listener, &shared))?
        };
        Ok(Self {
            shared,
            endpoint,
            accept: Some(accept),
        })
    }

    /// What to write to the endpoint file.
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// Address the server listens on.
    pub fn addr(&self) -> SocketAddr {
        SocketAddr::from((Ipv4Addr::LOCALHOST, self.endpoint.port))
    }

    /// Number of clients past the handshake.
    pub fn client_count(&self) -> usize {
        self.shared.hub().clients.len()
    }

    /// Remember `state` for clients that connect later and push it to everyone now.
    /// A client that cannot take it at once is dropped; the caller never waits.
    pub fn publish(&self, state: AppState) {
        let Ok(frame) = encode_message(&ServerMessage::State { state }) else {
            return;
        };
        let frame = Arc::new(frame);
        let mut hub = self.shared.hub();
        hub.state = state;
        hub.clients.retain(|c| {
            let sent = c.outbox.try_send(Arc::clone(&frame)).is_ok();
            if !sent {
                let _ = c.stream.shutdown(std::net::Shutdown::Both);
            }
            sent
        });
    }

    /// Stop accepting and close every connection.
    pub fn stop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        if let Some(accept) = self.accept.take() {
            let _ = accept.join();
        }
        for c in self.shared.hub().clients.drain(..) {
            let _ = c.stream.shutdown(std::net::Shutdown::Both);
        }
    }
}

impl Drop for LinkServer {
    fn drop(&mut self) {
        self.stop();
    }
}
