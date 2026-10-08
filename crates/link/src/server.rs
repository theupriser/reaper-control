//! The extension's side: accepts the app, pushes state, answers commands. Every socket
//! operation runs on its own thread, so a slow or broken client never holds up the caller.

mod acceptor;
mod command_handler;
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

use protocol::message::{PROTOCOL_VERSION, ServerMessage, encode_message};
use protocol::{Catalog, Live, WireEvent};

use crate::Endpoint;
use acceptor::Acceptor;
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
            protocol: PROTOCOL_VERSION,
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
                .spawn(move || Acceptor { listener, shared }.run())?
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
    pub fn address(&self) -> SocketAddr {
        SocketAddr::from((Ipv4Addr::LOCALHOST, self.endpoint.port))
    }

    /// Number of clients past the handshake.
    pub fn client_count(&self) -> usize {
        self.shared.hub().client_count()
    }

    /// Remember `live` for clients that connect later and push it to everyone now.
    /// A client that cannot take it at once is dropped; the caller never waits.
    pub fn publish(&self, live: Live) {
        let Ok(frame) = encode_message(&ServerMessage::Live(live.clone())) else {
            return;
        };
        self.shared.hub().broadcast(live, &Arc::new(frame));
    }

    /// Remember `catalog` for clients that connect later and push it to everyone now.
    pub fn publish_catalog(&self, catalog: Catalog) {
        let Ok(frame) = encode_message(&ServerMessage::Catalog(catalog.clone())) else {
            return;
        };
        self.shared
            .hub()
            .broadcast_catalog(catalog, &Arc::new(frame));
    }

    /// Number the event and push it to everyone. It is kept, so a client that reconnects
    /// can ask for what it missed.
    pub fn publish_event(&self, event: WireEvent) {
        self.shared.hub().broadcast_event(event);
    }

    /// Stop accepting and close every connection.
    pub fn stop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        if let Some(accept) = self.accept.take() {
            let _ = accept.join();
        }
        self.shared.hub().close_all();
    }
}

impl Drop for LinkServer {
    fn drop(&mut self) {
        self.stop();
    }
}
