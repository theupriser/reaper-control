use shared_kernel::Seconds;

use crate::MarkerName;

/// A marker in the project: a place to jump to, possibly carrying directives.
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    name: String,
    position: Seconds,
    parsed: MarkerName,
}

impl Cue {
    /// Parses the name once, here.
    pub fn new(name: impl Into<String>, position: Seconds) -> Self {
        let name = name.into();
        let parsed = MarkerName::parse(&name);
        Self {
            name,
            position,
            parsed,
        }
    }

    /// The marker name as written in REAPER.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Where the marker sits on the timeline.
    pub fn position(&self) -> Seconds {
        self.position
    }

    /// The name read through the grammar.
    pub fn parsed(&self) -> &MarkerName {
        &self.parsed
    }
}
