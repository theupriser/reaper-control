//! A recording driver for tests.

use std::sync::Mutex;

use protocol::Command;

use crate::driver::Driver;
use crate::driver_error::DriverError;

/// A driver that records commands, for tests; it can be told to be disconnected.
#[derive(Debug, Default)]
pub struct FakeDriver {
    sent: Mutex<Vec<Command>>,
    disconnected: Mutex<bool>,
}

impl FakeDriver {
    /// Makes every following `send` fail (or work again).
    pub fn set_disconnected(&self, disconnected: bool) {
        if let Ok(mut flag) = self.disconnected.lock() {
            *flag = disconnected;
        }
    }

    /// The commands sent so far, oldest first.
    #[must_use]
    pub fn sent(&self) -> Vec<Command> {
        self.sent
            .lock()
            .map(|sent| sent.clone())
            .unwrap_or_default()
    }
}

impl Driver for FakeDriver {
    fn send(&self, command: Command) -> Result<(), DriverError> {
        if self.disconnected.lock().is_ok_and(|flag| *flag) {
            return Err(DriverError::NotConnected);
        }
        if let Ok(mut sent) = self.sent.lock() {
            sent.push(command);
        }
        Ok(())
    }
}
