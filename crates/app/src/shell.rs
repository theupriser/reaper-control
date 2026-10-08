//! Tauri glue: holds the link, exposes `dispatch` and `current_view` to the UI and tells it when the view changes.

use tauri::{Emitter, Manager, State};

use std::sync::Arc;
use std::time::Duration;

use protocol::{Command, LinkView};

use crate::app_event::AppEvent;
use crate::command_bus::CommandBus;
use crate::endpoint_location::endpoint_file;
use crate::event_bus::EventBus;
use crate::health_monitor::HealthMonitor;
use crate::link_connection::LinkConnection;
use crate::queue_settings::QueueSettings;
use crate::system_clock::SystemClock;
use crate::system_process_check::SystemProcessCheck;

const VIEW_CHANGED: &str = "link-view";

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
    let result = tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let file = endpoint_file().ok_or("the home folder is unknown")?;
            let events = Arc::new(EventBus::default());
            events.subscribe(|event| match event {
                AppEvent::CommandRefused { command, error } => {
                    eprintln!("command {command:?} refused: {error}");
                }
                AppEvent::ExtensionRefused { id, reason } => {
                    eprintln!("command {id} refused by the extension: {reason}");
                }
                AppEvent::CommandSent(command) => eprintln!("command {command:?} sent"),
                AppEvent::CommandDropped(command) => {
                    eprintln!("command {command:?} dropped as a rapid repeat");
                }
                AppEvent::CommandQueueFull(command) => {
                    eprintln!("command {command:?} refused: too many commands are waiting");
                }
                AppEvent::CommandInvalid { command, refusal } => {
                    eprintln!("command {command:?} refused: {refusal}");
                }
                AppEvent::LinkHealthChanged { health } => eprintln!("link health: {health:?}"),
                AppEvent::CommandAcknowledged { id } => eprintln!("command {id} done"),
                AppEvent::CommandTimedOut { id, command } => {
                    eprintln!("command {id} ({command:?}) was not answered in time");
                }
                _ => {}
            });
            let clock = Arc::new(SystemClock::new());
            let health = Arc::new(HealthMonitor::new(
                events.clone(),
                clock.clone(),
                Arc::new(SystemProcessCheck),
            ));
            let link = Arc::new(LinkConnection::start(
                file,
                events.clone(),
                health.clone(),
                move |view| {
                    let _ = handle.emit(VIEW_CHANGED, view);
                },
            ));
            let bus = Arc::new(CommandBus::new(
                link.clone(),
                events,
                clock,
                QueueSettings::default(),
            ));
            let watched = Arc::clone(&bus);
            let ticker = std::thread::Builder::new()
                .name("command-timeouts".into())
                .spawn(move || {
                    loop {
                        std::thread::sleep(Duration::from_millis(500));
                        watched.expire();
                        health.tick();
                    }
                });
            if let Err(error) = ticker {
                eprintln!("command timeout thread failed to start: {error}");
            }
            app.manage(bus);
            app.manage(link);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![current_view, dispatch])
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("reaper control failed to start: {error}");
        std::process::exit(1);
    }
}
