//! Starting, stopping and restarting MIDI input as the settings change.

use std::sync::{Arc, Mutex};

use crate::clock::Clock;
use crate::event_bus::EventBus;
use crate::intent_dispatcher::IntentDispatcher;
use crate::midi_config::MidiConfig;
use crate::midi_listener::MidiListener;
use crate::midi_router::MidiRouter;
use crate::midi_source::MidiSource;

/// Owns the one running `MidiListener` and replaces it when the MIDI settings change.
pub struct MidiSwitch {
    source: Arc<dyn MidiSource>,
    intents: Arc<IntentDispatcher>,
    events: Arc<EventBus>,
    clock: Arc<dyn Clock>,
    running: Mutex<Option<MidiListener>>,
}

impl MidiSwitch {
    /// A switch with nothing running; call `apply` with the config to start.
    #[must_use]
    pub fn new(
        source: Arc<dyn MidiSource>,
        intents: Arc<IntentDispatcher>,
        events: Arc<EventBus>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            source,
            intents,
            events,
            clock,
            running: Mutex::new(None),
        }
    }

    /// Stops the listener that runs, closing its devices, and starts one for `config` when MIDI
    /// is enabled in it. A thread that cannot start is logged and leaves MIDI off.
    pub fn apply(&self, config: &MidiConfig) {
        let mut running = self
            .running
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Dropping the old listener joins its thread before the new one opens the same devices.
        *running = None;
        if !config.enabled {
            return;
        }
        let router = Arc::new(MidiRouter::new(
            config.clone(),
            Arc::clone(&self.intents),
            Arc::clone(&self.events),
            Arc::clone(&self.clock),
        ));
        match MidiListener::start(
            Arc::clone(&self.source),
            router,
            config.device_name.clone(),
            Arc::clone(&self.events),
        ) {
            Ok(listener) => *running = Some(listener),
            Err(error) => tracing::error!(%error, "midi thread failed to start"),
        }
    }

    /// Whether a listener is running now.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }
}
