use super::reset_fault;

#[test]
fn both_files_are_removed_and_other_files_stay() -> std::io::Result<()> {
    let directory = std::env::temp_dir().join(format!("reset-fault-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    for name in ["faulted", "running", "extension.log"] {
        std::fs::write(directory.join(name), "x")?;
    }
    reset_fault(&directory)?;
    assert!(!directory.join("faulted").exists());
    assert!(!directory.join("running").exists());
    assert!(directory.join("extension.log").exists());
    reset_fault(&directory)?;
    std::fs::remove_dir_all(&directory)
}
