use std::path::{Path, PathBuf};

/// What to start to get this app: the `.app` bundle the running program sits in on macOS, the
/// program itself elsewhere.
pub fn application_from(executable: &Path) -> PathBuf {
    executable
        .ancestors()
        .find(|ancestor| {
            ancestor
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("app"))
        })
        .unwrap_or(executable)
        .to_path_buf()
}

/// Tells the extension where the app is, in `file` next to `endpoint.json`, so a project can ask
/// for the app to be started. Only a packaged app does this: a development build needs its dev
/// server and would show an empty window when started on its own.
pub fn record_application_location(file: &Path, executable: &Path) -> std::io::Result<()> {
    let application = application_from(executable);
    let text = format!("{}\n", application.display());
    if std::fs::read_to_string(file).is_ok_and(|current| current == text) {
        return Ok(());
    }
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(file, text)
}
