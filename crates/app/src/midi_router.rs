//! From a raw MIDI message to a command on the bus.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::app_event::AppEvent;
use crate::clock::Clock;
use crate::event_bus::EventBus;
use crate::intent_dispatcher::IntentDispatcher;
use crate::midi_config::MidiConfig;
use crate::midi_debounce::MidiDebounce;
use crate::midi_message::MidiMessage;

/// Turns what MIDI devices send into commands: channel filter, global debounce, note mapping,
/// then the one `IntentDispatcher` every other controller uses.
pub struct MidiRouter {
    config: MidiConfig,
    intents: Arc<IntentDispatcher>,
    events: Arc<EventBus>,
    clock: Arc<dyn Clock>,
    // Boxed: the debounce table is 64 bytes.
    debounce: Mutex<Box<MidiDebounce>>,
}

impl MidiRouter {
    /// A router for `config` that hands intents to `intents`.
    #[must_use]
    pub fn new(
        config: MidiConfig,
        intents: Arc<IntentDispatcher>,
        events: Arc<EventBus>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let debounce = MidiDebounce::new(Duration::from_millis(config.debounce_milliseconds));
        Self {
            config,
            intents,
            events,
            clock,
            debounce: Mutex::new(Box::new(debounce)),
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
        let Some(intent) = self.config.notes.get(&note) else {
            return;
        };
        // The debounce holds nothing that a panic could leave half done, so a poisoned lock is used as is.
        let counts = self
            .debounce
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .allows(note, self.clock.now());
        if counts {
            // The bus and the dispatcher announce a refusal on the event bus; a note has nobody to return it to.
            let _ = self.intents.dispatch(*intent);
        }
    }
}

impl std::fmt::Debug for MidiRouter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("MidiRouter").finish_non_exhaustive()
    }
}
