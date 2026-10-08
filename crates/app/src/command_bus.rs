//! The single dispatch entry for commands.

use std::sync::Arc;

use protocol::Command;

use crate::app_event::AppEvent;
use crate::driver::Driver;
use crate::driver_error::DriverError;
use crate::event_bus::EventBus;

/// The one way a command leaves the app: UI, keyboard and MIDI all call `dispatch`.
pub struct CommandBus {
    driver: Arc<dyn Driver>,
    events: Arc<EventBus>,
}

impl CommandBus {
    /// A bus that sends through `driver` and reports on `events`.
    #[must_use]
    pub fn new(driver: Arc<dyn Driver>, events: Arc<EventBus>) -> Self {
        Self { driver, events }
    }

    /// Sends `command` and publishes whether it went out or was refused.
    pub fn dispatch(&self, command: Command) -> Result<(), DriverError> {
        match self.driver.send(command.clone()) {
            Ok(()) => {
                self.events.publish(&AppEvent::CommandSent(command));
                Ok(())
            }
            Err(error) => {
                self.events
                    .publish(&AppEvent::CommandRefused { command, error });
                Err(error)
            }
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
