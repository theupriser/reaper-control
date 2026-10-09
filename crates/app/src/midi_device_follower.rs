//! Keeping the right MIDI devices open as they come and go.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::app_event::AppEvent;
use crate::event_bus::EventBus;
use crate::midi_router::MidiRouter;
use crate::midi_source::{MidiConnection, MidiSource};

/// Opens the wanted device (every device when none is named, as v1 did) and closes what was
/// unplugged. `poll` is one look at the device list; the listener calls it every second.
pub struct MidiDeviceFollower {
    source: Arc<dyn MidiSource>,
    router: Arc<MidiRouter>,
    wanted: Option<String>,
    events: Arc<EventBus>,
    open: BTreeMap<String, MidiConnection>,
}

impl MidiDeviceFollower {
    /// A follower that opens nothing until the first `poll`.
    #[must_use]
    pub fn new(
        source: Arc<dyn MidiSource>,
        router: Arc<MidiRouter>,
        wanted: Option<String>,
        events: Arc<EventBus>,
    ) -> Self {
        Self {
            source,
            router,
            wanted,
            events,
            open: BTreeMap::new(),
        }
    }

    /// Compares the devices present with the open ones and announces a change.
    pub fn poll(&mut self) {
        // A listing that failed says nothing about the devices, so nothing is closed.
        let Some(names) = self.source.device_names() else {
            return;
        };
        let before = self.open.len();
        self.open.retain(|name, _| names.contains(name));
        let mut changed = self.open.len() != before;
        for name in names {
            let wanted = self.wanted.as_ref().is_none_or(|only| *only == name);
            if wanted
                && !self.open.contains_key(&name)
                && let Some(connection) = self.source.open(&name, Arc::clone(&self.router))
            {
                self.open.insert(name, connection);
                changed = true;
            }
        }
        if changed {
            self.events.publish(&AppEvent::MidiDevicesChanged {
                devices: self.open.keys().cloned().collect(),
            });
        }
    }
}
