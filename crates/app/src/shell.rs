//! Tauri glue: holds the link, exposes `dispatch` and `current_view` to the UI and tells it when the view changes.

use tauri::{Emitter, Manager, State};

use protocol::{Command, LinkView};

use crate::endpoint_location::endpoint_file;
use crate::link_connection::LinkConnection;

const VIEW_CHANGED: &str = "link-view";

#[tauri::command]
fn current_view(link: State<'_, LinkConnection>) -> LinkView {
    link.view()
}

#[tauri::command]
fn dispatch(link: State<'_, LinkConnection>, command: Command) -> Result<(), String> {
    link.send(command).map_err(|error| error.to_string())
}

/// Starts the window and blocks until it closes.
pub fn run() {
    let result = tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let file = endpoint_file().ok_or("the home folder is unknown")?;
            app.manage(LinkConnection::start(file, move |view| {
                let _ = handle.emit(VIEW_CHANGED, view);
            }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![current_view, dispatch])
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("reaper control failed to start: {error}");
        std::process::exit(1);
    }
}
