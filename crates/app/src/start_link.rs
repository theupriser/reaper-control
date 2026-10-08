//! Starts the app's side of the link: the real one, or the simulator (`RC2_SIMULATOR` set).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use protocol::LinkView;

use crate::driver::Driver;
use crate::event_bus::EventBus;
use crate::health_monitor::HealthMonitor;
use crate::link_connection::LinkConnection;
use crate::link_view_source::LinkViewSource;
use crate::simulator::{Simulator, sample_project};

/// How often the simulated time moves on while the app runs.
const SIMULATOR_STEP: Duration = Duration::from_millis(50);

/// Returns what commands go through and where the view is read. With `simulated` the project is
/// the sample project and its clock follows the wall clock.
pub fn start_link(
    simulated: bool,
    endpoint_file: PathBuf,
    events: Arc<EventBus>,
    health: Arc<HealthMonitor>,
    on_change: impl Fn(LinkView) + Send + 'static,
) -> (Arc<dyn Driver>, Arc<dyn LinkViewSource>) {
    if !simulated {
        let link = Arc::new(LinkConnection::start(
            endpoint_file,
            events,
            health,
            on_change,
        ));
        return (link.clone(), link);
    }
    let simulator = Arc::new(Simulator::start(
        sample_project(),
        events,
        health,
        on_change,
    ));
    let running = Arc::clone(&simulator);
    let clock = std::thread::Builder::new()
        .name("simulator-clock".into())
        .spawn(move || {
            loop {
                std::thread::sleep(SIMULATOR_STEP);
                running.advance(SIMULATOR_STEP);
            }
        });
    if let Err(error) = clock {
        tracing::error!(%error, "simulator clock thread failed to start");
    }
    (simulator.clone(), simulator)
}
