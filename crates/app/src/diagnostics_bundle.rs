//! One zip with what a person needs to send when something went wrong.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::diagnostics_error::DiagnosticsError;

/// The extension's files that go into the bundle. `endpoint.json` is left out: it holds the
/// link's token.
const EXTENSION_FILES: [&str; 4] = [
    "extension.log",
    "journal.log",
    "journal.previous.log",
    "faulted",
];

/// Where the files for the bundle are.
pub struct DiagnosticsBundle {
    logs: Option<PathBuf>,
    extension: Option<PathBuf>,
    config: Option<PathBuf>,
}

impl DiagnosticsBundle {
    /// A bundle of the app's log folder, the extension's folder and the config file.
    #[must_use]
    pub fn new(logs: Option<PathBuf>, extension: Option<PathBuf>, config: Option<PathBuf>) -> Self {
        Self {
            logs,
            extension,
            config,
        }
    }

    /// Writes `reaper-control-diagnostics-<seconds>.zip` into `directory` and returns its path.
    /// `link_status` goes into `about.txt` with the versions.
    ///
    /// # Errors
    /// A [`DiagnosticsError`] when the folder or the zip cannot be written. Missing source files
    /// are skipped: the bundle is most needed when things are broken.
    pub fn export(&self, directory: &Path, link_status: &str) -> Result<PathBuf, DiagnosticsError> {
        std::fs::create_dir_all(directory)?;
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        let target = directory.join(format!("reaper-control-diagnostics-{seconds}.zip"));
        let mut zip = ZipWriter::new(std::fs::File::create(&target)?);
        let options = match zip::DateTime::try_from(plain_now()) {
            Ok(stamp) => SimpleFileOptions::default().last_modified_time(stamp),
            Err(_) => SimpleFileOptions::default(),
        };
        let about = format!(
            "Reaper Control {}\nsystem: {} {}\nlink: {link_status}\n",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
        );
        add(&mut zip, "about.txt", about.as_bytes(), options)?;
        if let Some(config) = &self.config {
            add_file(&mut zip, "config.json", config, options)?;
        }
        if let Some(logs) = &self.logs {
            for entry in std::fs::read_dir(logs).into_iter().flatten().flatten() {
                let name = format!("logs/{}", entry.file_name().to_string_lossy());
                add_file(&mut zip, &name, &entry.path(), options)?;
            }
        }
        if let Some(extension) = &self.extension {
            for name in EXTENSION_FILES {
                add_file(
                    &mut zip,
                    &format!("extension/{name}"),
                    &extension.join(name),
                    options,
                )?;
            }
        }
        zip.finish()?;
        Ok(target)
    }
}

fn add_file(
    zip: &mut ZipWriter<std::fs::File>,
    name: &str,
    source: &Path,
    options: SimpleFileOptions,
) -> Result<(), DiagnosticsError> {
    match std::fs::read(source) {
        Ok(bytes) => add(zip, name, &bytes, options),
        Err(_) => Ok(()),
    }
}

fn add(
    zip: &mut ZipWriter<std::fs::File>,
    name: &str,
    bytes: &[u8],
    options: SimpleFileOptions,
) -> Result<(), DiagnosticsError> {
    zip.start_file(name, options)?;
    zip.write_all(bytes)?;
    Ok(())
}

fn plain_now() -> time::PrimitiveDateTime {
    let now = time::OffsetDateTime::now_utc();
    time::PrimitiveDateTime::new(now.date(), now.time())
}
