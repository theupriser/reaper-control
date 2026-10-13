use std::fs;
use std::path::PathBuf;

use super::Launcher;

/// How long the app gets to connect by itself before it is started. An app that is already
/// running reconnects within a couple of seconds (and on macOS `open` does not start a second copy).
const GRACE_SECONDS: f64 = 2.0;

/// Starts the app when the open project asks for it and no app connects. It asks once per project
/// (a project that was left and opened again asks again), and never when an app is, or was while
/// this project was open, connected.
pub struct Autostarter {
    application_file: PathBuf,
    launcher: Box<dyn Launcher>,
    project: Option<String>,
    waiting_since: Option<f64>,
    settled: bool,
}

impl Autostarter {
    /// `application_file` holds the path of the app, written by the app itself.
    pub fn new(application_file: PathBuf, launcher: Box<dyn Launcher>) -> Self {
        Self {
            application_file,
            launcher,
            project: None,
            waiting_since: None,
            settled: false,
        }
    }

    /// Looks at the state once per tick. Returns a line for the log when it started the app, or
    /// failed to.
    pub fn observe(
        &mut self,
        project_id: &str,
        autostart: bool,
        clients: usize,
        now: f64,
    ) -> Option<String> {
        if project_id.is_empty() {
            return None;
        }
        if self.project.as_deref() != Some(project_id) {
            self.project = Some(project_id.to_owned());
            self.waiting_since = None;
            self.settled = false;
        }
        if self.settled {
            return None;
        }
        if clients > 0 {
            self.settled = true;
            return None;
        }
        if !autostart {
            self.waiting_since = None;
            return None;
        }
        let since = *self.waiting_since.get_or_insert(now);
        if now - since < GRACE_SECONDS {
            return None;
        }
        self.settled = true;
        Some(self.start())
    }

    fn start(&mut self) -> String {
        let Ok(text) = fs::read_to_string(&self.application_file) else {
            return "autostart: the app has not told where it is; open the installed app once"
                .into();
        };
        let application = PathBuf::from(text.trim());
        match self.launcher.launch(&application) {
            Ok(()) => format!("autostart: started {}", application.display()),
            Err(reason) => format!("autostart: could not start the app: {reason}"),
        }
    }
}

impl std::fmt::Debug for Autostarter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Autostarter")
            .field("project", &self.project)
            .field("settled", &self.settled)
            .finish_non_exhaustive()
    }
}
