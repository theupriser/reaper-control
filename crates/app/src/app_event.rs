//! What the app announces about itself.

use protocol::{Command, EventRecord};

use crate::command_refusal::CommandRefusal;
use crate::driver_error::DriverError;
use crate::link_health::LinkHealth;

/// Something that happened inside the app, for anyone who listens (log, UI, tests).
#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    /// A command went to the driver.
    CommandSent(Command),
    /// The driver could not take a command.
    CommandRefused {
        /// The command that was not sent.
        command: Command,
        /// Why.
        error: DriverError,
    },
    /// An identical command was sent a moment ago, so this one was dropped as a rapid repeat.
    CommandDropped(Command),
    /// Too many commands were waiting for REAPER, so this one was not sent.
    CommandQueueFull(Command),
    /// The command can never be right, so it was not sent.
    CommandInvalid {
        /// The command.
        command: Command,
        /// What is wrong with it.
        refusal: CommandRefusal,
    },
    /// The extension answered a command and it was done.
    CommandAcknowledged {
        /// Id the link gave the command when it was sent.
        id: u64,
    },
    /// The extension did not answer a command in time.
    CommandTimedOut {
        /// Id the link gave the command when it was sent.
        id: u64,
        /// What was sent.
        command: Command,
    },
    /// The handshake with the extension is done.
    LinkConnected {
        /// Version of the extension build.
        extension_version: String,
    },
    /// The connection to the extension is gone.
    LinkLost,
    /// The health of the link changed.
    LinkHealthChanged {
        /// The new health.
        health: LinkHealth,
    },
    /// The extension received a command and refused it.
    ExtensionRefused {
        /// Id the link gave the command when it was sent.
        id: u64,
        /// Why, as the extension says.
        reason: String,
    },
    /// Something happened in the performance.
    PerformanceEvent(EventRecord),
    /// The app was away too long to be caught up on the performance events.
    EventsMissed {
        /// The oldest event id the extension still holds.
        oldest_available: u64,
    },
}
