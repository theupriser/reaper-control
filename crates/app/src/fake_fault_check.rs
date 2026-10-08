//! A fault check that tests control.

use std::sync::Mutex;

use crate::fault_check::FaultCheck;

/// Answers whatever the test last set; no fault at first.
#[derive(Debug, Default)]
pub struct FakeFaultCheck {
    reason: Mutex<Option<String>>,
}

impl FakeFaultCheck {
    /// Sets the fault that counts as reported.
    pub fn set_reason(&self, reason: Option<&str>) {
        if let Ok(mut current) = self.reason.lock() {
            *current = reason.map(str::to_owned);
        }
    }
}

impl FaultCheck for FakeFaultCheck {
    fn reason(&self) -> Option<String> {
        self.reason.lock().ok().and_then(|current| current.clone())
    }
}
