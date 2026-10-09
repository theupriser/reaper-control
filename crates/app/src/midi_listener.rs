//! The thread that follows the MIDI devices.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::event_bus::EventBus;
use crate::midi_device_follower::MidiDeviceFollower;
use crate::midi_router::MidiRouter;
use crate::midi_source::MidiSource;

/// `midir` reports no hotplug, so a thread looks at the device list every second. Dropping the
/// listener stops the thread and closes the devices it had open.
pub struct MidiListener {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl MidiListener {
    /// Starts the thread.
    ///
    /// # Errors
    /// The error of the operating system when the thread cannot start.
    pub fn start(
        source: Arc<dyn MidiSource>,
        router: Arc<MidiRouter>,
        wanted: Option<String>,
        events: Arc<EventBus>,
    ) -> std::io::Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = Arc::clone(&stop);
        let thread = std::thread::Builder::new()
            .name("midi-input".into())
            .spawn(move || {
                let mut follower = MidiDeviceFollower::new(source, router, wanted, events);
                while !stopping.load(Ordering::SeqCst) {
                    follower.poll();
                    std::thread::park_timeout(Duration::from_secs(1));
                }
            })?;
        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for MidiListener {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            thread.thread().unpark();
            // A panic in the thread has already been reported by the panic hook.
            let _ = thread.join();
        }
    }
}
