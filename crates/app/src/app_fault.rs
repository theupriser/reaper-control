//! The last internal error of the app (a panic), kept so the window can show a safe state instead of freezing.

use std::sync::Mutex;

/// Holds the text of the last panic.
#[derive(Default)]
pub struct AppFault {
    message: Mutex<Option<String>>,
}

impl AppFault {
    /// Remembers the panic and returns the text for the window.
    pub fn record(&self, panic: &str) -> String {
        let text = format!(
            "The app hit an internal error and may not respond. REAPER keeps playing. Restart the app. ({panic})"
        );
        if let Ok(mut message) = self.message.lock() {
            *message = Some(text.clone());
        }
        text
    }

    /// The text to show, once a panic was recorded.
    #[must_use]
    pub fn message(&self) -> Option<String> {
        self.message.lock().ok().and_then(|message| message.clone())
    }
}
