//! Listening to the MIDI device with `midir`, and following it when it is unplugged or plugged in.

use std::sync::Arc;
use std::time::Duration;

use midir::{MidiInput, MidiInputConnection};

use crate::app_event::AppEvent;
use crate::event_bus::EventBus;
use crate::midi_router::MidiRouter;

const CLIENT_NAME: &str = "Reaper Control";

/// Keeps one MIDI input open: the configured device, or the first one found. `midir` reports no
/// hotplug, so a thread looks at the device list every second.
pub struct MidiListener;

impl MidiListener {
    /// Starts the thread; it ends with the process.
    ///
    /// # Errors
    /// The error of the operating system when the thread cannot start.
    pub fn start(
        router: Arc<MidiRouter>,
        wanted: Option<String>,
        events: Arc<EventBus>,
    ) -> std::io::Result<()> {
        std::thread::Builder::new()
            .name("midi-input".into())
            .spawn(move || {
                let mut open: Option<(String, MidiInputConnection<()>)> = None;
                loop {
                    let names = device_names();
                    if open.as_ref().is_some_and(|(name, _)| !names.contains(name)) {
                        open = None;
                        events.publish(&AppEvent::MidiDeviceChanged { device: None });
                    }
                    if open.is_none()
                        && let Some(name) = names
                            .iter()
                            .find(|name| wanted.as_ref().is_none_or(|wanted| wanted == *name))
                    {
                        open = connect(name, &router);
                        if open.is_some() {
                            events.publish(&AppEvent::MidiDeviceChanged {
                                device: Some(name.clone()),
                            });
                        }
                    }
                    std::thread::sleep(Duration::from_secs(1));
                }
            })
            .map(|_| ())
    }
}

/// The names of the input devices present now.
#[must_use]
pub fn device_names() -> Vec<String> {
    let Ok(input) = MidiInput::new(CLIENT_NAME) else {
        return Vec::new();
    };
    input
        .ports()
        .iter()
        .filter_map(|port| input.port_name(port).ok())
        .collect()
}

fn connect(name: &str, router: &Arc<MidiRouter>) -> Option<(String, MidiInputConnection<()>)> {
    let input = MidiInput::new(CLIENT_NAME).ok()?;
    let port = input
        .ports()
        .into_iter()
        .find(|port| input.port_name(port).is_ok_and(|found| found == name))?;
    let router = Arc::clone(router);
    let connection = input
        .connect(&port, "input", move |_, bytes, ()| router.handle(bytes), ())
        .ok()?;
    Some((name.to_owned(), connection))
}
