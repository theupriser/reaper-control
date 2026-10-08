//! The real stats source.

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use protocol::SystemStats;

use crate::stats_source::StatsSource;

const BYTES_PER_MEGABYTE: u64 = 1024 * 1024;

fn megabytes(bytes: u64) -> u32 {
    u32::try_from(bytes / BYTES_PER_MEGABYTE).unwrap_or(u32::MAX)
}

/// Reads CPU and memory with `sysinfo`.
pub struct SysinfoStatsSource {
    system: System,
}

impl Default for SysinfoStatsSource {
    fn default() -> Self {
        let mut system = System::new();
        system.refresh_cpu_usage();
        Self { system }
    }
}

impl StatsSource for SysinfoStatsSource {
    fn read(&mut self) -> SystemStats {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        let app = self
            .system
            .process(Pid::from_u32(std::process::id()))
            .map_or(0, |process| process.memory());
        let reaper = self
            .system
            .processes()
            .values()
            .filter(|process| {
                process
                    .name()
                    .to_string_lossy()
                    .to_lowercase()
                    .trim_end_matches(".exe")
                    == "reaper"
            })
            .map(|process| process.memory())
            .reduce(|total, memory| total + memory);
        SystemStats {
            machine_cpu_percent: self.system.global_cpu_usage(),
            memory_used_megabytes: megabytes(self.system.used_memory()),
            memory_total_megabytes: megabytes(self.system.total_memory()),
            app_memory_megabytes: megabytes(app),
            reaper_memory_megabytes: reaper.map(megabytes),
        }
    }
}
