/// Identity of a setlist inside its project.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SetlistId(String);

impl SetlistId {
    /// Wraps an id.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The id text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
