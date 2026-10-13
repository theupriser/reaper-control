//! The app tells the extension where it is, so a project can ask for it to be started.

use std::path::{Path, PathBuf};

use app::{application_from, record_application_location};

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("rc2-application-{}-{name}", std::process::id()))
}

#[test]
fn a_macos_program_is_found_by_its_bundle() {
    let executable = Path::new("/Applications/Reaper Control.app/Contents/MacOS/app");
    assert_eq!(
        application_from(executable),
        PathBuf::from("/Applications/Reaper Control.app")
    );
}

#[test]
fn a_windows_program_is_the_program_itself() {
    let executable = Path::new("C:/Program Files/Reaper Control/reaper-control.exe");
    assert_eq!(application_from(executable), executable);
}

#[test]
fn the_location_is_written_once_and_replaced_when_the_app_moves() -> std::io::Result<()> {
    let directory = scratch("write");
    let file = directory.join("RC2").join("application");
    record_application_location(
        &file,
        Path::new("/one/Reaper Control.app/Contents/MacOS/app"),
    )?;
    assert_eq!(std::fs::read_to_string(&file)?, "/one/Reaper Control.app\n");
    record_application_location(
        &file,
        Path::new("/two/Reaper Control.app/Contents/MacOS/app"),
    )?;
    assert_eq!(std::fs::read_to_string(&file)?, "/two/Reaper Control.app\n");
    std::fs::remove_dir_all(&directory)
}
