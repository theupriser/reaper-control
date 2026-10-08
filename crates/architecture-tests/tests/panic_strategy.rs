//! The extension must unwind on panic (SPEC S-9.1, ADR-009): with `panic = "abort"` one panic
//! would kill REAPER in the middle of a show.

use cargo_metadata::MetadataCommand;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn no_profile_aborts_on_panic() -> TestResult {
    let metadata = MetadataCommand::new().exec()?;
    let manifests = std::iter::once(metadata.workspace_root.join("Cargo.toml")).chain(
        metadata
            .workspace_packages()
            .into_iter()
            .map(|p| p.manifest_path.clone()),
    );
    for manifest in manifests {
        let text = std::fs::read_to_string(&manifest)?;
        for line in text.lines() {
            let compact: String = line.split_whitespace().collect();
            assert!(
                !compact.starts_with("panic=\"abort\"") && !compact.starts_with("panic='abort'"),
                "{manifest} sets panic = \"abort\": `{line}`"
            );
        }
    }
    Ok(())
}
