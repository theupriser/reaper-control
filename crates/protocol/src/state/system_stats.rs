use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// How busy the machine is, for the stats popover. Memory is in megabytes.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
pub struct SystemStats {
    /// CPU use of the whole machine, 0 to 100.
    pub machine_cpu_percent: f32,
    /// Memory in use on the machine.
    pub memory_used_megabytes: u32,
    /// Memory installed in the machine.
    pub memory_total_megabytes: u32,
    /// Memory used by this app.
    pub app_memory_megabytes: u32,
    /// Memory used by REAPER, or none when no REAPER process was found.
    pub reaper_memory_megabytes: Option<u32>,
}
