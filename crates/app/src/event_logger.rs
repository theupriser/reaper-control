//! Turns what the app announces into log lines, at a level that fits how much it matters on stage.

use tracing::{debug, info, warn};

use crate::app_event::AppEvent;

/// Writes one log line for the event.
pub fn log_event(event: &AppEvent) {
    match event {
        AppEvent::CommandSent(command) => debug!(?command, "command sent"),
        AppEvent::CommandAcknowledged { id } => debug!(id, "command done"),
        AppEvent::CommandRefused { command, error } => {
            warn!(?command, %error, "command refused");
        }
        AppEvent::CommandDropped(command) => {
            debug!(?command, "command dropped as a rapid repeat");
        }
        AppEvent::CommandQueueFull(command) => {
            warn!(?command, "command refused: too many commands are waiting");
        }
        AppEvent::CommandInvalid { command, refusal } => {
            warn!(?command, %refusal, "command refused");
        }
        AppEvent::CommandTimedOut { id, command } => {
            warn!(id, ?command, "command was not answered in time");
        }
        AppEvent::ExtensionRefused { id, reason } => {
            warn!(id, %reason, "command refused by the extension");
        }
        AppEvent::LinkConnected { extension_version } => {
            info!(%extension_version, "link connected");
        }
        AppEvent::LinkLost => warn!("link lost"),
        AppEvent::LinkHealthChanged { health } => info!(?health, "link health changed"),
        AppEvent::MidiActivity {
            channel,
            note,
            velocity,
        } => debug!(channel, note, velocity, "midi note"),
        AppEvent::MidiDevicesChanged { devices } => info!(?devices, "midi devices changed"),
        AppEvent::PerformanceEvent(record) => info!(?record, "performance event"),
        AppEvent::EventsMissed { oldest_available } => {
            warn!(oldest_available, "performance events were missed");
        }
    }
}
