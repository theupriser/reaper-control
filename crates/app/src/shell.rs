//! Tauri glue: holds the state and exposes `dispatch` and `current_state` to the UI.

use std::sync::Mutex;

use tauri::State;

use crate::fake_performance::{self, AppState, Command};

type Shared = Mutex<AppState>;

#[tauri::command]
fn current_state(shared: State<'_, Shared>) -> Result<AppState, String> {
    shared.lock().map(|s| *s).map_err(|e| e.to_string())
}

#[tauri::command]
fn dispatch(shared: State<'_, Shared>, command: Command) -> Result<AppState, String> {
    let mut guard = shared.lock().map_err(|e| e.to_string())?;
    *guard = fake_performance::dispatch(*guard, command);
    Ok(*guard)
}

/// Starts the window and blocks until it closes.
pub fn run() {
    let result = tauri::Builder::default()
        .manage(Shared::default())
        .invoke_handler(tauri::generate_handler![current_state, dispatch])
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("reaper control failed to start: {error}");
        std::process::exit(1);
    }
}
