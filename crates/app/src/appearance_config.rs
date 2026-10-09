//! The look as stored in the config file.

use serde::{Deserialize, Serialize};

/// The themes the Settings screen offers.
pub const THEMES: [&str; 3] = ["dark", "stage-dark", "light"];
/// The densities the Settings screen offers.
pub const DENSITIES: [&str; 2] = ["comfortable", "compact"];
/// The touch sizes the Settings screen offers.
pub const TOUCH_SIZES: [&str; 2] = ["normal", "large"];

/// How the screens look; the UI applies it as soon as it is read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    /// One of [`THEMES`].
    pub theme: String,
    /// One of [`DENSITIES`].
    pub density: String,
    /// One of [`TOUCH_SIZES`].
    pub touch: String,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            density: "comfortable".into(),
            touch: "normal".into(),
        }
    }
}

impl AppearanceConfig {
    /// The first value outside its allowed set, as a message.
    #[must_use]
    pub fn problem(&self) -> Option<String> {
        [
            ("theme", &self.theme, THEMES.as_slice()),
            ("density", &self.density, DENSITIES.as_slice()),
            ("touch size", &self.touch, TOUCH_SIZES.as_slice()),
        ]
        .into_iter()
        .find(|(_, value, allowed)| !allowed.contains(&value.as_str()))
        .map(|(name, value, allowed)| {
            format!("{name} {value:?} is not one of {}", allowed.join(", "))
        })
    }
}
