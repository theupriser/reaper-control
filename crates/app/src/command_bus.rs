//! The single dispatch entry for commands.

use std::sync::{Arc, Mutex};

use protocol::Command;

use crate::app_event::AppEvent;
use crate::clock::Clock;
use crate::command_check::check;
use crate::command_queue::CommandQueue;
use crate::dispatch_error::DispatchError;
use crate::driver::Driver;
use crate::driver_error::DriverError;
use crate::event_bus::EventBus;
use crate::queue_rejection::QueueRejection;
use crate::queue_settings::QueueSettings;

/// The one way a command leaves the app: UI, keyboard and MIDI all call `dispatch`. Commands
/// leave in the order they arrive; rapid repeats are dropped, and a command REAPER does not
/// answer (also when the link is lost meanwhile: it may or may not have run) is reported.
pub struct CommandBus {
    driver: Arc<dyn Driver>,
    events: Arc<EventBus>,
    queue: Arc<Mutex<CommandQueue>>,
}

impl CommandBus {
    /// A bus that sends through `driver` and reports on `events`. It listens there for the
    /// extension's answers.
    #[must_use]
    pub fn new(
        driver: Arc<dyn Driver>,
        events: Arc<EventBus>,
        clock: Arc<dyn Clock>,
        settings: QueueSettings,
    ) -> Self {
        let queue = Arc::new(Mutex::new(CommandQueue::new(clock, settings)));
        let listening = Arc::clone(&queue);
        events.subscribe(move |event| {
            let Ok(mut queue) = listening.lock() else {
                return;
            };
            match event {
                AppEvent::CommandAcknowledged { id } | AppEvent::ExtensionRefused { id, .. } => {
                    queue.acknowledge(*id);
                }
                _ => {}
            }
        });
        Self {
            driver,
            events,
            queue,
        }
    }

    /// Uses these queue limits from now on.
    pub fn apply(&self, settings: QueueSettings) {
        if let Ok(mut queue) = self.queue.lock() {
            queue.apply(settings);
        }
    }

    /// Sends `command` and publishes whether it went out, was dropped as a repeat or was refused.
    /// A command that can never be right is refused first.
    pub fn dispatch(&self, command: Command) -> Result<(), DispatchError> {
        self.expire();
        if let Err(refusal) = check(&command) {
            self.events
                .publish(&AppEvent::CommandInvalid { command, refusal });
            return Err(refusal.into());
        }
        let sent = {
            let mut queue = self.queue.lock().map_err(|_| DriverError::NotConnected)?;
            match queue.admit(&command) {
                Err(QueueRejection::Repeat) => Err(AppEvent::CommandDropped(command.clone())),
                Err(QueueRejection::Full) => Err(AppEvent::CommandQueueFull(command.clone())),
                Ok(()) => self
                    .driver
                    .send(command.clone())
                    .map(|id| {
                        queue.track(id, command.clone());
                    })
                    .map_err(|error| AppEvent::CommandRefused {
                        command: command.clone(),
                        error,
                    }),
            }
        };
        match sent {
            Ok(()) => {
                self.events.publish(&AppEvent::CommandSent(command));
                Ok(())
            }
            Err(event) => {
                self.events.publish(&event);
                match event {
                    AppEvent::CommandDropped(_) => Ok(()),
                    AppEvent::CommandRefused { error, .. } => Err(error.into()),
                    _ => Err(DispatchError::QueueFull),
                }
            }
        }
    }

    /// Reports the commands the extension has not answered in time. Call it regularly.
    pub fn expire(&self) {
        let expired = self
            .queue
            .lock()
            .map(|mut queue| queue.expire())
            .unwrap_or_default();
        for pending in expired {
            self.events.publish(&AppEvent::CommandTimedOut {
                id: pending.id,
                command: pending.command,
            });
        }
    }
}

impl std::fmt::Debug for CommandBus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("CommandBus").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
