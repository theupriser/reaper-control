//! How well the link to the extension is doing.

use crate::link_cause::LinkCause;

/// The health of the link. It only gets better through a new connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkHealth {
    /// The extension answers.
    Connected,
    /// Connected, but the extension has been silent for a moment.
    Degraded,
    /// The connection is gone and the app is trying again; it is not yet known why.
    Lost,
    /// Lost for long enough (or known to be hopeless) that the cause is named.
    Dead(LinkCause),
}
