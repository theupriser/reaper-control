use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn it_reads_the_reason_and_sees_no_fault_without_a_file() -> TestResult {
    let directory = std::env::temp_dir().join(format!("rc2-fault-check-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    let check = FaultFileCheck::new(directory.join("faulted"));
    assert_eq!(check.reason(), None);
    std::fs::write(directory.join("faulted"), "safe mode\n")?;
    assert_eq!(check.reason().as_deref(), Some("safe mode"));
    std::fs::write(directory.join("faulted"), "  \n")?;
    assert_eq!(check.reason(), None);
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}
