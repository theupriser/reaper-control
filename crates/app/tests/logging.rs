//! The log reaches a file in the folder, and an unusable folder does not stop the app.

use app::app_event::AppEvent;
use app::event_logger::log_event;
use app::logging::Logging;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn events_reach_the_log_file_and_a_bad_folder_only_costs_the_file() -> TestResult {
    let directory = std::env::temp_dir().join(format!("app-logging-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;

    let logging = Logging::start(Some(directory.clone()));
    log_event(&AppEvent::LinkConnected {
        extension_version: "9.9.9".into(),
    });
    log_event(&AppEvent::LinkLost);
    drop(logging);

    let mut text = String::new();
    for entry in std::fs::read_dir(&directory)? {
        text.push_str(&std::fs::read_to_string(entry?.path())?);
    }
    assert!(text.contains("link connected"), "got {text:?}");
    assert!(text.contains("9.9.9"), "got {text:?}");
    assert!(text.contains("WARN"), "got {text:?}");
    std::fs::remove_dir_all(&directory)?;

    let file_in_the_way =
        std::env::temp_dir().join(format!("app-logging-file-{}", std::process::id()));
    std::fs::write(&file_in_the_way, "not a folder")?;
    let _unusable = Logging::start(Some(file_in_the_way.join("logs")));
    std::fs::remove_file(&file_in_the_way)?;
    Ok(())
}
