//! Tauri glue: holds the link, exposes `dispatch` and `current_view` to the UI and tells it when the view changes.

use tauri::{Emitter, Manager, State};

use std::sync::Arc;
use std::time::Duration;

use protocol::{Command, LinkView, Phase};

use crate::app_event::AppEvent;
use crate::command_bus::CommandBus;
use crate::config_location::config_file;
use crate::config_store::ConfigStore;
use crate::endpoint_location::endpoint_file;
use crate::event_bus::EventBus;
use crate::health_monitor::HealthMonitor;
use crate::link_connection::LinkConnection;
use crate::midi_listener::MidiListener;
use crate::midi_router::MidiRouter;
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
                AppEvent::MidiActivity {
                    channel,
                    note,
                    velocity,
                } => eprintln!("midi note {note} velocity {velocity} on channel {channel}"),
                AppEvent::MidiDeviceChanged { device } => eprintln!("midi device: {device:?}"),
                AppEvent::CommandAcknowledged { id } => eprintln!("command {id} done"),
                AppEvent::CommandTimedOut { id, command } => {
                    eprintln!("command {id} ({command:?}) was not answered in time");
                }
                _ => {}
            });
            let config = config_file()
                .map(|file| ConfigStore::new(file).load())
                .transpose()
                .unwrap_or_else(|error| {
                    eprintln!("{error}; using the default settings");
                    None
                })
                .unwrap_or_default();
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
                events.clone(),
                clock.clone(),
                config.queue.settings(),
            ));
            if config.midi.enabled {
                let running = link.clone();
                let router = Arc::new(MidiRouter::new(
                    config.midi.clone(),
                    bus.clone(),
                    events.clone(),
                    clock,
                    move || {
                        running.view().live.is_some_and(|live| {
                            matches!(
                                live.phase,
                                Phase::Playing | Phase::CountingIn | Phase::HandingOver
                            )
                        })
                    },
                ));
                if let Err(error) =
                    MidiListener::start(router, config.midi.device_name.clone(), events)
                {
                    eprintln!("midi thread failed to start: {error}");
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
                eprintln!("command timeout thread failed to start: {error}");
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
                eprintln!("link health thread failed to start: {error}");
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
