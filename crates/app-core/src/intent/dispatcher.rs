//! The one path an intent takes from any controller to the command bus.

use std::sync::Arc;

use protocol::LinkView;

use crate::command::CommandBus;
use crate::event::AppEvent;
use crate::event::EventBus;
use crate::intent::Intent;
use crate::intent::IntentError;
use crate::intent::IntentTranslator;

/// Keyboard, MIDI and remote controllers all call `dispatch`: the intent is judged against the
/// current state, then sent through the same `CommandBus` as the UI's commands.
pub struct IntentDispatcher {
    bus: Arc<CommandBus>,
    events: Arc<EventBus>,
    view: Box<dyn Fn() -> LinkView + Send + Sync>,
}

impl IntentDispatcher {
    /// A dispatcher that reads the current state through `view`.
    #[must_use]
    pub fn new(
        bus: Arc<CommandBus>,
        events: Arc<EventBus>,
        view: impl Fn() -> LinkView + Send + Sync + 'static,
    ) -> Self {
        Self {
            bus,
            events,
            view: Box::new(view),
        }
    }

    /// Translates and sends `intent`; a refusal is also announced on the event bus.
    pub fn dispatch(&self, intent: Intent) -> Result<(), IntentError> {
        match IntentTranslator::translate(intent, &(self.view)()) {
            Ok(command) => Ok(self.bus.dispatch(command)?),
            Err(refusal) => {
                self.events
                    .publish(&AppEvent::IntentRefused { intent, refusal });
                Err(refusal.into())
            }
        }
    }
}

impl std::fmt::Debug for IntentDispatcher {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("IntentDispatcher")
            .finish_non_exhaustive()
    }
}
