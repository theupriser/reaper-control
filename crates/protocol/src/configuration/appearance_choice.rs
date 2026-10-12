use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// How the screens look.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct AppearanceChoice {
    /// The colours: dark, stage-dark or light.
    pub theme: String,
    /// How tight the layout is: comfortable or compact.
    pub density: String,
    /// How large the touch targets are: normal or large.
    pub touch: String,
}
