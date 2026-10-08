//! Joins the link server to the timer loop: commands come in on the main thread's tick, state goes out.

mod bridge_error;
mod queued_commands;

pub use bridge_error::BridgeError;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};

use link::LinkServer;
use protocol::message::Outcome;
use protocol::{Command, Live};
use reaper_port::ReaperPort;

use crate::log::Log;
use crate::timer_loop::TimerLoop;
use queued_commands::QueuedCommands;

/// The running link and the endpoint file that tells the app where to find it.
pub struct LinkBridge {
    server: LinkServer,
    commands: Receiver<Command>,
    endpoint_file: PathBuf,
    published: Option<Live>,
    sequence: u64,
    published_revisions: Option<(u64, u64)>,
}

impl LinkBridge {
    /// Starts the server and writes `endpoint.json` into `directory`.
    pub fn start(directory: &Path, log: &Log) -> Result<Self, BridgeError> {
        let (sender, commands) = channel();
        let server = LinkServer::start(env!("CARGO_PKG_VERSION"), QueuedCommands::new(sender))?;
        let endpoint_file = directory.join("endpoint.json");
        server.endpoint().write(&endpoint_file)?;
        log.line(&format!(
            "link listening on port {}",
            server.endpoint().port
        ));
        Ok(Self {
            server,
            commands,
            endpoint_file,
            published: None,
            sequence: 0,
            published_revisions: None,
        })
    }

    /// Carries out the queued commands, then pushes the state when it changed.
    pub fn pump<Port: ReaperPort>(&mut self, timer_loop: &mut TimerLoop<Port>, log: &Log) {
        while let Ok(command) = self.commands.try_recv() {
            if let Outcome::Rejected { reason } = timer_loop.link_command(command) {
                log.line(&format!("link command {command:?} refused: {reason}"));
            }
        }
        let catalog = timer_loop.catalog();
        let revisions = (catalog.revision, catalog.setlist_revision);
        if self.published_revisions != Some(revisions) {
            self.server.publish_catalog(catalog.clone());
            self.published_revisions = Some(revisions);
        }
        let live = timer_loop.live();
        if self.published.as_ref() != Some(&live) {
            self.sequence += 1;
            self.published = Some(live.clone());
            self.server.publish(Live {
                sequence: self.sequence,
                timestamp: timer_loop.now(),
                ..live
            });
        }
    }
}

impl Drop for LinkBridge {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.endpoint_file);
    }
}

impl std::fmt::Debug for LinkBridge {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LinkBridge")
            .field("endpoint_file", &self.endpoint_file)
            .finish_non_exhaustive()
    }
}
