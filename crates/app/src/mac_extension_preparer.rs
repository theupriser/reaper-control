//! Signs the copied extension ad hoc and clears its quarantine flag (measured in ADR-006: a
//! quarantined dylib is blocked by Gatekeeper, one without the flag loads).

use std::path::Path;
use std::process::Command;

use crate::extension_preparer::ExtensionPreparer;

/// Prepares the extension for macOS.
#[derive(Clone, Copy, Debug, Default)]
pub struct MacExtensionPreparer;

impl ExtensionPreparer for MacExtensionPreparer {
    fn prepare(&self, library: &Path) -> Result<(), String> {
        run(Command::new("codesign")
            .args(["--force", "-s", "-"])
            .arg(library))?;
        // Fails when the file was never quarantined; the check below is what counts.
        let _ = Command::new("xattr")
            .args(["-d", "com.apple.quarantine"])
            .arg(library)
            .output();
        let still_quarantined = Command::new("xattr")
            .args(["-p", "com.apple.quarantine"])
            .arg(library)
            .output()
            .map_err(|error| error.to_string())?
            .status
            .success();
        if still_quarantined {
            return Err("the quarantine flag could not be removed".into());
        }
        Ok(())
    }
}

fn run(command: &mut Command) -> Result<(), String> {
    let output = command.output().map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
