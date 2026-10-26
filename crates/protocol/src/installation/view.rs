use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::WizardStep;

/// What the first-run wizard shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct InstallationView {
    /// The four steps in order.
    pub steps: Vec<WizardStep>,
    /// The REAPER resource folder that was found, to copy or reveal.
    pub folder: String,
    /// Whether the "Install" button can do something now.
    pub can_install: bool,
    /// Whether everything is in place and the extension answers.
    pub complete: bool,
    /// Whether an older extension is installed and the app has a newer one to put in its place.
    pub update_available: bool,
    /// Whether REAPER runs now; an update then closes it first.
    pub reaper_running: bool,
}
