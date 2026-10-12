use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What is wrong with the link, for the sidebar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct LinkProblem {
    /// A few words naming the problem.
    pub message: String,
    /// The installed extension is older than the one that ships with the app, so the way out is the
    /// setup wizard.
    pub extension_outdated: bool,
}
