//! Where the app writes its log: standard error and a daily file that keeps the last two weeks.

use std::path::PathBuf;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::MakeWriterExt;

const FILTER_VARIABLE: &str = "RC2_LOG";
const DAYS_KEPT: usize = 14;

/// The running log. Keep it alive until the app ends: dropping it writes out what is still waiting.
pub struct Logging {
    _guard: Option<WorkerGuard>,
}

impl Logging {
    /// Starts logging at the level in `RC2_LOG` (default `info`) to standard error and, when the
    /// folder can be written, to a daily file in it. Without a usable folder it logs to standard
    /// error only: the app never stops for a log.
    #[must_use]
    pub fn start(directory: Option<PathBuf>) -> Self {
        let filter =
            EnvFilter::try_from_env(FILTER_VARIABLE).unwrap_or_else(|_| EnvFilter::new("info"));
        let appender = directory.and_then(|directory| {
            RollingFileAppender::builder()
                .rotation(Rotation::DAILY)
                .filename_prefix("reaper-control")
                .filename_suffix("log")
                .max_log_files(DAYS_KEPT)
                .build(directory)
                .map_err(|error| eprintln!("no log file ({error}); logging to standard error"))
                .ok()
        });
        let (guard, installed) = match appender {
            Some(appender) => {
                let (writer, guard) = tracing_appender::non_blocking(appender);
                let installed = tracing_subscriber::fmt()
                    .with_env_filter(filter)
                    .with_ansi(false)
                    .with_writer(std::io::stderr.and(writer))
                    .try_init();
                (Some(guard), installed)
            }
            None => (
                None,
                tracing_subscriber::fmt()
                    .with_env_filter(filter)
                    .with_writer(std::io::stderr)
                    .try_init(),
            ),
        };
        if let Err(error) = installed {
            eprintln!("logging was already set up: {error}");
        }
        Self { _guard: guard }
    }
}
