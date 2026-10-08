use super::FaultFile;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn temp_dir(name: &str) -> Result<std::path::PathBuf, std::io::Error> {
    let directory = std::env::temp_dir().join(format!("rc2-fault-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}

#[test]
fn a_reported_fault_can_be_read_back_and_cleared() -> TestResult {
    let directory = temp_dir("report")?;
    let file = FaultFile::new(directory.join("faulted"));
    file.report("safe mode")?;
    assert_eq!(
        std::fs::read_to_string(directory.join("faulted"))?,
        "safe mode"
    );
    file.clear();
    assert!(!directory.join("faulted").exists());
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}

#[test]
fn clearing_when_nothing_was_reported_is_fine() -> TestResult {
    let directory = temp_dir("none")?;
    FaultFile::new(directory.join("faulted")).clear();
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}
