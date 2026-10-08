use std::path::PathBuf;

use reaper_medium::ControlSurface;

use crate::fault::Fault;
use crate::log::Log;

const HEARTBEAT_EVERY: u64 = 300;

/// REAPER calls `run` about 30 times a second on its main thread. It is the extension's timer.
#[derive(Debug)]
pub(super) struct Surface {
    fault: &'static Fault,
    log: &'static Log,
    #[cfg_attr(not(feature = "fault-injection"), allow(dead_code))]
    directory: PathBuf,
    ticks: u64,
}

impl Surface {
    pub(super) fn new(fault: &'static Fault, log: &'static Log, directory: PathBuf) -> Self {
        Self {
            fault,
            log,
            directory,
            ticks: 0,
        }
    }

    fn tick(&mut self) {
        self.ticks += 1;
        if self.ticks % HEARTBEAT_EVERY == 1 {
            self.log.line(&format!("tick {}", self.ticks));
        }
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
