use std::path::PathBuf;
use std::time::Instant;

use reaper_medium::ControlSurface;

use super::reaper_rs_adapter::ReaperRsAdapter;
use crate::fault::Fault;
use crate::fault_file::FaultFile;
use crate::link_bridge::LinkBridge;
use crate::log::Log;
use crate::tick_watchdog::{TickWatchdog, Verdict};
use crate::timer_loop::TimerLoop;

const HEARTBEAT_EVERY: u64 = 300;

/// REAPER calls `run` about 30 times a second on its main thread. It is the extension's timer.
#[derive(Debug)]
pub(super) struct Surface {
    fault: &'static Fault,
    faults: &'static FaultFile,
    log: &'static Log,
    #[cfg_attr(not(feature = "fault-injection"), allow(dead_code))]
    directory: PathBuf,
    timer_loop: TimerLoop<ReaperRsAdapter>,
    watchdog: TickWatchdog,
    link: Option<LinkBridge>,
    #[cfg(feature = "probe")]
    probe: super::probe::Probe,
    ticks: u64,
}

impl Surface {
    pub(super) fn new(
        fault: &'static Fault,
        log: &'static Log,
        faults: &'static FaultFile,
        directory: PathBuf,
        adapter: ReaperRsAdapter,
    ) -> Self {
        let link = match LinkBridge::start(&directory, log) {
            Ok(link) => Some(link),
            Err(error) => {
                log.line(&format!("link not started: {error}"));
                None
            }
        };
        Self {
            fault,
            faults,
            log,
            #[cfg(feature = "probe")]
            probe: super::probe::Probe::new(directory.clone()),
            directory,
            timer_loop: TimerLoop::new(adapter),
            watchdog: TickWatchdog::new(),
            link,
            ticks: 0,
        }
    }

    fn judge(&mut self, elapsed: std::time::Duration) {
        match self.watchdog.judge(elapsed) {
            Verdict::Within => {}
            Verdict::OverBudget => self.log.line(&format!(
                "slow tick: {elapsed:?} (over budget so far: {})",
                self.watchdog.over_budget()
            )),
            Verdict::Disable => {
                self.log.line(&format!(
                    "WATCHDOG: ticks keep stalling ({elapsed:?}); extension disabled"
                ));
                self.fault.trip();
                let _ = self
                    .faults
                    .report("REAPER kept stalling, so the extension switched itself off");
            }
        }
    }

    fn tick(&mut self) {
        self.ticks += 1;
        if self.ticks % HEARTBEAT_EVERY == 1 {
            self.log.line(&format!(
                "tick {}, slowest so far {:?}, over budget {}",
                self.ticks,
                self.watchdog.slowest(),
                self.watchdog.over_budget()
            ));
        }
        #[cfg(feature = "probe")]
        self.probe.run(&mut self.timer_loop, self.log);
        let started = Instant::now();
        self.timer_loop.tick();
        if let Some(link) = &mut self.link {
            link.pump(&mut self.timer_loop, self.log);
        }
        #[cfg(feature = "fault-injection")]
        if self.directory.join("slow-main").exists() {
            // every tick stays slow while the trigger file exists
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        self.judge(started.elapsed());
        #[cfg(feature = "fault-injection")]
        if std::fs::remove_file(self.directory.join("panic-main")).is_ok() {
            self.log.line("injecting a panic in the tick");
            #[allow(clippy::panic)] // fault injection is the point of this feature
            {
                panic!("injected panic in tick");
            }
        }
    }
}

impl ControlSurface for Surface {
    fn run(&mut self) {
        let fault = self.fault;
        fault.guard(|| self.tick());
    }
}
