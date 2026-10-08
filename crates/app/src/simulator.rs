//! The extension without REAPER: the same timer loop and performance rules over `FakeReaper`,
//! with time that only moves when told to (SPEC §14, WP 4.11).

mod sample_project;
mod simulator_state;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use protocol::{Command, LinkView};
use reaper_port::FakeReaper;

pub use sample_project::sample_project;
use simulator_state::{STEP_SECONDS, SimulatorState};

use crate::driver::Driver;
use crate::driver_error::DriverError;
use crate::event_bus::EventBus;
use crate::health_monitor::HealthMonitor;
use crate::link_pipeline::LinkPipeline;
use crate::link_view_source::LinkViewSource;

/// Plays a project in memory and reports like the link would, so the whole app can be run and
/// tested without REAPER. `advance` moves its time; nothing else does.
pub struct Simulator {
    state: Mutex<SimulatorState>,
    view: Arc<Mutex<LinkView>>,
}

impl Simulator {
    /// Starts the simulated extension over `reaper`; it is connected and has published its
    /// catalog and state when this returns. Announcements go to `events`, every new view to
    /// `on_change`.
    pub fn start(
        reaper: FakeReaper,
        events: Arc<EventBus>,
        health: Arc<HealthMonitor>,
        on_change: impl Fn(LinkView) + Send + 'static,
    ) -> Self {
        let view = Arc::new(Mutex::new(LinkView::default()));
        let pipeline = LinkPipeline::new(Arc::clone(&view), events, health, on_change);
        Self {
            state: Mutex::new(SimulatorState::start(reaper, pipeline)),
            view,
        }
    }

    /// Lets `span` of simulated time pass, in the extension's 50 ms steps.
    pub fn advance(&self, span: Duration) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        let mut remaining = span.as_secs_f64();
        while remaining > 0.0 {
            let step = remaining.min(STEP_SECONDS);
            state.step(step);
            remaining -= step;
        }
    }
}

impl Driver for Simulator {
    fn send(&self, command: Command) -> Result<u64, DriverError> {
        let mut state = self.state.lock().map_err(|_| DriverError::NotConnected)?;
        Ok(state.command(command))
    }
}

impl LinkViewSource for Simulator {
    fn view(&self) -> LinkView {
        self.view
            .lock()
            .map(|view| view.clone())
            .unwrap_or_default()
    }
}
