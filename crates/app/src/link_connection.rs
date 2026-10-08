//! The app's side of the link, kept alive for the whole run.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use link::{ClientConfig, Endpoint, LinkClient, SendError};
use protocol::{Command, LinkView};

use crate::driver::Driver;
use crate::driver_error::DriverError;
use crate::event_bus::EventBus;
use crate::health_monitor::HealthMonitor;
use crate::link_pipeline::LinkPipeline;
use crate::link_view_source::LinkViewSource;

/// The app's one connection to the extension, and what the UI shows about it.
pub struct LinkConnection {
    client: LinkClient,
    view: Arc<Mutex<LinkView>>,
}

impl LinkConnection {
    /// Starts connecting to the extension that `endpoint_file` points at. A missing, stale or
    /// unreadable file is the same as REAPER not running: the client keeps trying.
    /// `on_change` is called with the new view after every change, from the link thread; what
    /// the link reports is also announced on `events`.
    pub fn start(
        endpoint_file: PathBuf,
        events: Arc<EventBus>,
        health: Arc<HealthMonitor>,
        on_change: impl Fn(LinkView) + Send + 'static,
    ) -> Self {
        let config = ClientConfig::new(move || Endpoint::read(&endpoint_file).ok());
        let (client, link_events) = LinkClient::start(config);
        let view = Arc::new(Mutex::new(LinkView::default()));
        let mut pipeline = LinkPipeline::new(Arc::clone(&view), events, health, on_change);
        let spawned = thread::Builder::new()
            .name("link-view".into())
            .spawn(move || {
                for event in link_events {
                    if !pipeline.deliver(event) {
                        return;
                    }
                }
            });
        if let Err(error) = spawned {
            tracing::error!(%error, "link view thread failed to start");
        }
        Self { client, view }
    }

    /// Sends a command to the extension; refused while the link is down.
    pub fn send(&self, command: Command) -> Result<(), SendError> {
        self.client.send(command).map(|_| ())
    }
}

impl LinkViewSource for LinkConnection {
    fn view(&self) -> LinkView {
        self.view
            .lock()
            .map(|view| view.clone())
            .unwrap_or_default()
    }
}

impl Driver for LinkConnection {
    fn send(&self, command: Command) -> Result<u64, DriverError> {
        self.client
            .send(command)
            .map_err(|_| DriverError::NotConnected)
    }
}
