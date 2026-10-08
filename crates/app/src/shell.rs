//! Tauri glue: holds the link, exposes `dispatch` and `current_view` to the UI and tells it when the view changes.

use tauri::{Emitter, Manager, State};

use std::sync::Arc;

use protocol::{Command, LinkView};

use crate::app_event::AppEvent;
use crate::command_bus::CommandBus;
use crate::endpoint_location::endpoint_file;
use crate::event_bus::EventBus;
use crate::link_connection::LinkConnection;

const VIEW_CHANGED: &str = "link-view";

#[tauri::command]
fn current_view(link: State<'_, Arc<LinkConnection>>) -> LinkView {
    link.view()
}

#[tauri::command]
fn dispatch(bus: State<'_, CommandBus>, command: Command) -> Result<(), String> {
    bus.dispatch(command).map_err(|error| error.to_string())
}

/// Starts the window and blocks until it closes.
pub fn run() {
    let result = tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let file = endpoint_file().ok_or("the home folder is unknown")?;
            let link = Arc::new(LinkConnection::start(file, move |view| {
                let _ = handle.emit(VIEW_CHANGED, view);
            }));
            let events = Arc::new(EventBus::default());
            events.subscribe(|event| {
                if let AppEvent::CommandRefused { command, error } = event {
                    eprintln!("command {command:?} refused: {error}");
                }
            });
            app.manage(CommandBus::new(link.clone(), events));
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
