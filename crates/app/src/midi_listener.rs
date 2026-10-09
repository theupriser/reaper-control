//! The thread that follows the MIDI devices.

use std::sync::Arc;
use std::time::Duration;

use crate::event_bus::EventBus;
use crate::midi_device_follower::MidiDeviceFollower;
use crate::midi_router::MidiRouter;
use crate::midi_source::MidiSource;

/// `midir` reports no hotplug, so a thread looks at the device list every second.
pub struct MidiListener;

impl MidiListener {
    /// Starts the thread; it ends with the process.
    ///
    /// # Errors
    /// The error of the operating system when the thread cannot start.
    pub fn start(
        source: Arc<dyn MidiSource>,
        router: Arc<MidiRouter>,
        wanted: Option<String>,
        events: Arc<EventBus>,
    ) -> std::io::Result<()> {
        std::thread::Builder::new()
            .name("midi-input".into())
            .spawn(move || {
                let mut follower = MidiDeviceFollower::new(source, router, wanted, events);
                loop {
                    follower.poll();
                    std::thread::sleep(Duration::from_secs(1));
                }
            })
            .map(|_| ())
    }
}
