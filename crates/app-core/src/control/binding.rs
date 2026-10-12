use crate::control::ControlSource;
use crate::intent::Intent;

/// One control and what pressing it asks for. Every controller turns its presses into the same
/// `Intent`, so the one `IntentDispatcher` checks them all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlBinding {
    /// The control.
    pub source: ControlSource,
    /// What pressing it asks for.
    pub intent: Intent,
}

impl ControlBinding {
    /// The intent bound to `source` among `bindings`, if any.
    #[must_use]
    pub fn intent_for(bindings: &[Self], source: ControlSource) -> Option<Intent> {
        bindings
            .iter()
            .find(|binding| binding.source == source)
            .map(|binding| binding.intent)
    }
}
