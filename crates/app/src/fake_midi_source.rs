//! MIDI devices that tests plug in and out.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::midi_router::MidiRouter;
use crate::midi_source::{MidiConnection, MidiSource};

/// For tests: a list of device names, and the routers of the devices that are open.
#[derive(Debug, Default)]
pub struct FakeMidiSource {
    plugged: Mutex<Option<Vec<String>>>,
    open: Arc<Mutex<BTreeMap<String, Arc<MidiRouter>>>>,
}

/// Removes the device from the open ones when dropped, like closing it.
struct FakeConnection {
    name: String,
    open: Arc<Mutex<BTreeMap<String, Arc<MidiRouter>>>>,
}

impl Drop for FakeConnection {
    fn drop(&mut self) {
        if let Ok(mut open) = self.open.lock() {
            open.remove(&self.name);
        }
    }
}

impl FakeMidiSource {
    /// Sets the devices that are present; `None` makes listing fail.
    pub fn plug(&self, names: Option<&[&str]>) {
        if let Ok(mut plugged) = self.plugged.lock() {
            *plugged = names.map(|names| names.iter().map(|name| (*name).to_owned()).collect());
        }
    }

    /// The names of the devices that are open now.
    #[must_use]
    pub fn open_devices(&self) -> Vec<String> {
        self.open
            .lock()
            .map(|open| open.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Sends bytes as if the open device `name` had played them; false when it is not open.
    #[must_use]
    pub fn send(&self, name: &str, bytes: &[u8]) -> bool {
        let router = self
            .open
            .lock()
            .ok()
            .and_then(|open| open.get(name).cloned());
        router.map(|router| router.handle(bytes)).is_some()
    }
}

impl MidiSource for FakeMidiSource {
    fn device_names(&self) -> Option<Vec<String>> {
        self.plugged.lock().ok().and_then(|plugged| plugged.clone())
    }

    fn open(&self, name: &str, router: Arc<MidiRouter>) -> Option<MidiConnection> {
        self.open.lock().ok()?.insert(name.to_owned(), router);
        Some(Box::new(FakeConnection {
            name: name.to_owned(),
            open: Arc::clone(&self.open),
        }))
    }
}
