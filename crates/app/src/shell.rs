//! Tauri glue: holds the link, exposes `dispatch` and `current_view` to the UI and tells it when the view changes.

use tauri::{Emitter, Manager, State};

use std::sync::Arc;
use std::time::Duration;

use protocol::{Command, LinkView};

use crate::command_bus::CommandBus;
use crate::config_location::{config_file, log_directory};
use crate::config_store::ConfigStore;
use crate::endpoint_location::{endpoint_file, fault_file};
use crate::event_bus::EventBus;
use crate::event_logger::log_event;
use crate::fault_file_check::FaultFileCheck;
use crate::health_monitor::HealthMonitor;
use crate::intent_dispatcher::IntentDispatcher;
use crate::link_connection::LinkConnection;
use crate::logging::Logging;
use crate::metered_driver::MeteredDriver;
use crate::midi_listener::MidiListener;
use crate::midi_router::MidiRouter;
use crate::notice_for_event::notice_for_event;
use crate::system_clock::SystemClock;
use crate::system_process_check::SystemProcessCheck;

const VIEW_CHANGED: &str = "link-view";
const NOTICE: &str = "notice";

#[tauri::command]
fn current_view(link: State<'_, Arc<LinkConnection>>) -> LinkView {
    link.view()
}

#[tauri::command]
fn dispatch(bus: State<'_, Arc<CommandBus>>, command: Command) -> Result<(), String> {
    bus.dispatch(command).map_err(|error| error.to_string())
}

/// Starts the window and blocks until it closes.
pub fn run() {
    let logging = Logging::start(log_directory());
    let result = tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let file = endpoint_file().ok_or("the home folder is unknown")?;
            let events = Arc::new(EventBus::default());
            events.subscribe(log_event);
            let notices = handle.clone();
            events.subscribe(move |event| {
                if let Some(notice) = notice_for_event(event) {
                    let _ = notices.emit(NOTICE, notice);
                }
            });
            let config = config_file()
                .map(|file| ConfigStore::new(file).load())
                .transpose()
                .unwrap_or_else(|error| {
                    tracing::warn!(%error, "using the default settings");
                    None
                })
                .unwrap_or_default();
            let clock = Arc::new(SystemClock::new());
            let health = Arc::new(HealthMonitor::new(
                events.clone(),
                clock.clone(),
                Arc::new(SystemProcessCheck),
                Arc::new(FaultFileCheck::new(
                    fault_file().ok_or("the home folder is unknown")?,
                )),
            ));
            let link = Arc::new(LinkConnection::start(
                file,
                events.clone(),
                health.clone(),
                move |view| {
                    let _ = handle.emit(VIEW_CHANGED, view);
                },
            ));
            let driver = Arc::new(MeteredDriver::new(link.clone(), clock.clone(), &events));
            let bus = Arc::new(CommandBus::new(
                driver,
                events.clone(),
                clock.clone(),
                config.queue.settings(),
            ));
            if config.midi.enabled {
                let watched = link.clone();
                let intents = Arc::new(IntentDispatcher::new(
                    bus.clone(),
                    events.clone(),
                    move || watched.view(),
                ));
                let router = Arc::new(MidiRouter::new(
                    config.midi.clone(),
                    intents,
                    events.clone(),
                    clock,
                ));
                if let Err(error) =
                    MidiListener::start(router, config.midi.device_name.clone(), events)
                {
                    tracing::error!(%error, "midi thread failed to start");
                }
            }
            let watched = Arc::clone(&bus);
            let ticker = std::thread::Builder::new()
                .name("command-timeouts".into())
                .spawn(move || {
                    loop {
                        std::thread::sleep(Duration::from_millis(500));
                        watched.expire();
                    }
                });
            if let Err(error) = ticker {
                tracing::error!(%error, "command timeout thread failed to start");
            }
            let health_ticker = std::thread::Builder::new()
                .name("link-health".into())
                .spawn(move || {
                    loop {
                        std::thread::sleep(Duration::from_secs(1));
                        health.tick();
                    }
                });
            if let Err(error) = health_ticker {
                tracing::error!(%error, "link health thread failed to start");
            }
            app.manage(bus);
            app.manage(link);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![current_view, dispatch])
        .run(tauri::generate_context!());
    if let Err(error) = result {
        tracing::error!(%error, "reaper control failed to start");
        drop(logging);
        std::process::exit(1);
    }
}
