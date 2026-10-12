use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What the app makes of a marker or region name: the answer the Help screen shows while you type.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
pub struct MarkerReading {
    /// The name holds `!1008` or `!hardstop`.
    pub hard_stop: bool,
    /// The song length from `!length:N`, in seconds.
    pub length_seconds: Option<f64>,
    /// The tempo from `!bpm:N`.
    pub tempo_bpm: Option<f64>,
    /// The name holds only commands, so the timeline does not draw it.
    pub hidden: bool,
}
