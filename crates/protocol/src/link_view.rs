use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{AppState, Catalog, LinkStatus};

/// Everything the UI shows about the link: the connection and the last state the extension pushed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct LinkView {
    /// Connection to the extension.
    pub status: LinkStatus,
    /// Last pushed state; the default while nothing has been pushed.
    pub state: AppState,
    /// Last pushed catalog; empty while nothing has been pushed.
    pub catalog: Catalog,
}

impl Default for LinkView {
    fn default() -> Self {
        Self {
            status: LinkStatus::NotRunning,
            state: AppState::default(),
            catalog: Catalog::default(),
        }
    }
}
