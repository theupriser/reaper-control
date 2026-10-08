//! Joins the link server to the timer loop: commands come in on the main thread's tick, state goes out.

mod bridge_error;

pub use bridge_error::BridgeError;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};

use link::LinkServer;
use protocol::Command;
use protocol::message::Outcome;
use reaper_port::ReaperPort;

use crate::journal::Journal;
use crate::log::Log;
use timer_loop::{QueuedCommands, StatePublisher, TimerLoop};

/// The running link and the endpoint file that tells the app where to find it.
pub struct LinkBridge {
    server: LinkServer,
    commands: Receiver<Command>,
    endpoint_file: PathBuf,
    journal: Journal,
    publisher: StatePublisher,
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
            journal: Journal::new(directory.join("journal.log")),
            publisher: StatePublisher::default(),
        })
    }

    /// Carries out the queued commands, then pushes the state when it changed.
    pub fn pump<Port: ReaperPort>(&mut self, timer_loop: &mut TimerLoop<Port>, log: &Log) {
        while let Ok(command) = self.commands.try_recv() {
            let description = format!("{command:?}");
            if let Outcome::Rejected { reason } = timer_loop.link_command(command) {
                log.line(&format!("link command {description} refused: {reason}"));
            }
        }
        let journal = &self.journal;
        self.publisher
            .publish(timer_loop, &self.server, |event| journal.record(event));
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
