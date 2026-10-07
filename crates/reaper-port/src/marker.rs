use shared_kernel::Seconds;

/// A marker of the project as REAPER reports it.
#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    /// The marker's name, directives included.
    pub name: String,
    /// Where it sits.
    pub position: Seconds,
}
