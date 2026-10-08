//! From a raw MIDI message to a command on the bus.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::app_event::AppEvent;
use crate::clock::Clock;
use crate::command_bus::CommandBus;
use crate::event_bus::EventBus;
use crate::midi_config::MidiConfig;
use crate::midi_debounce::MidiDebounce;
use crate::midi_message::MidiMessage;

/// Turns what MIDI devices send into commands: channel filter, global debounce, note mapping,
/// then the one `CommandBus::dispatch` every other source uses.
pub struct MidiRouter {
    config: MidiConfig,
    bus: Arc<CommandBus>,
    events: Arc<EventBus>,
    clock: Arc<dyn Clock>,
    playing: Box<dyn Fn() -> bool + Send + Sync>,
    debounce: Mutex<MidiDebounce>,
}

impl MidiRouter {
    /// A router for `config`; `playing` tells whether the performance runs, for the play toggle.
    #[must_use]
    pub fn new(
        config: MidiConfig,
        bus: Arc<CommandBus>,
        events: Arc<EventBus>,
        clock: Arc<dyn Clock>,
        playing: impl Fn() -> bool + Send + Sync + 'static,
    ) -> Self {
        let debounce = MidiDebounce::new(Duration::from_millis(config.debounce_milliseconds));
        Self {
            config,
            bus,
            events,
            clock,
            playing: Box::new(playing),
            debounce: Mutex::new(debounce),
        }
    }

    /// Handles the bytes one device sent. Anything but a note-on, a release (velocity 0), a note
    /// on another channel than the configured one, a repeat inside the debounce window and an
    /// unmapped note does nothing.
    pub fn handle(&self, bytes: &[u8]) {
        let Some(MidiMessage::NoteOn {
            channel,
            note,
            velocity,
        }) = MidiMessage::parse(bytes)
        else {
            return;
        };
        if velocity == 0 || self.config.channel.is_some_and(|only| only != channel) {
            return;
        }
        self.events.publish(&AppEvent::MidiActivity {
            channel,
            note,
            velocity,
        });
        let Some(action) = self.config.notes.get(&note) else {
            return;
        };
        let counts = self
            .debounce
            .lock()
            .map(|mut debounce| debounce.allows(note, self.clock.now()))
            .unwrap_or(false);
        if counts {
            // The bus announces a refusal on the event bus; a note has nobody to return it to.
            let _ = self.bus.dispatch(action.command((self.playing)()));
        }
    }
}

impl std::fmt::Debug for MidiRouter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("MidiRouter").finish_non_exhaustive()
    }
}
