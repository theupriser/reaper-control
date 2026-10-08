use super::SafeModeMarker;
use crate::start_up::StartUp;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn temp_dir(name: &str) -> Result<std::path::PathBuf, std::io::Error> {
    let directory = std::env::temp_dir().join(format!("rc2-marker-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}

#[test]
fn a_first_start_writes_the_marker_and_runs_normally() -> TestResult {
    let directory = temp_dir("first")?;
    let path = directory.join("running");
    let StartUp::Normal(marker) = SafeModeMarker::claim(&path)? else {
        return Err("expected a normal start".into());
    };
    assert!(path.exists());
    assert_eq!(marker.path(), path);
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}

#[test]
fn a_clean_shutdown_lets_the_next_start_run_normally() -> TestResult {
    let directory = temp_dir("clean")?;
    let path = directory.join("running");
    let StartUp::Normal(marker) = SafeModeMarker::claim(&path)? else {
        return Err("expected a normal start".into());
    };
    marker.release();
    assert!(!path.exists());
    assert!(matches!(SafeModeMarker::claim(&path)?, StartUp::Normal(_)));
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}

#[test]
fn a_marker_left_behind_means_safe_mode_and_is_kept() -> TestResult {
    let directory = temp_dir("crash")?;
    let path = directory.join("running");
    std::fs::write(&path, b"1234")?;
    assert!(matches!(SafeModeMarker::claim(&path)?, StartUp::SafeMode));
    assert!(path.exists(), "safe mode must not clear the marker itself");
    assert!(matches!(SafeModeMarker::claim(&path)?, StartUp::SafeMode));
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}

#[test]
fn a_marker_that_cannot_be_written_is_an_error_not_a_normal_start() -> TestResult {
    let directory = temp_dir("missing")?;
    let path = directory.join("no-such-folder").join("running");
    assert!(SafeModeMarker::claim(&path).is_err());
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}
