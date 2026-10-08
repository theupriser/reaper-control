//! The diagnostics bundle holds the logs, the journal, the config and the versions, and never the token.

use std::io::Read;

use app::diagnostics_bundle::DiagnosticsBundle;
use app::journal_import::JournalImport;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn scratch(name: &str) -> Result<std::path::PathBuf, std::io::Error> {
    let directory = std::env::temp_dir().join(format!("app-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}

#[test]
fn the_bundle_has_the_files_that_exist_and_leaves_the_token_out() -> TestResult {
    let root = scratch("bundle")?;
    let logs = root.join("logs");
    let extension = root.join("RC2");
    std::fs::create_dir_all(&logs)?;
    std::fs::create_dir_all(&extension)?;
    std::fs::write(logs.join("reaper-control.2026-10-08.log"), "app line\n")?;
    std::fs::write(extension.join("journal.log"), "{\"event\":1}\n")?;
    std::fs::write(extension.join("endpoint.json"), "{\"token\":\"secret\"}")?;
    std::fs::write(root.join("config.json"), "{}")?;

    let bundle =
        DiagnosticsBundle::new(Some(logs), Some(extension), Some(root.join("config.json")));
    let target = bundle.export(&root.join("out"), "NotRunning")?;

    let mut archive = zip::ZipArchive::new(std::fs::File::open(&target)?)?;
    let mut names: Vec<String> = archive.file_names().map(str::to_owned).collect();
    names.sort();
    assert_eq!(
        names,
        [
            "about.txt",
            "config.json",
            "extension/journal.log",
            "logs/reaper-control.2026-10-08.log"
        ]
    );
    let year = archive
        .by_name("about.txt")?
        .last_modified()
        .map_or(0, |stamp| stamp.year());
    assert!(year >= 2026, "entries carry the real date, got {year}");
    let mut about = String::new();
    archive.by_name("about.txt")?.read_to_string(&mut about)?;
    assert!(about.contains("link: NotRunning"), "got {about:?}");
    std::fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn the_bundle_still_works_when_nothing_exists() -> TestResult {
    let root = scratch("bundle-empty")?;
    let bundle = DiagnosticsBundle::new(None, Some(root.join("nothing")), None);
    let target = bundle.export(&root, "Lost")?;
    let archive = zip::ZipArchive::new(std::fs::File::open(&target)?)?;
    assert_eq!(archive.len(), 1);
    std::fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn the_import_reads_only_whole_new_lines_and_follows_a_replaced_journal() -> TestResult {
    let root = scratch("journal")?;
    let file = root.join("journal.log");
    let import = JournalImport::new(file.clone());
    assert_eq!(import.import(), 0, "no journal yet");

    std::fs::write(&file, "one\ntwo\nthr")?;
    assert_eq!(import.import(), 2, "the half line waits");
    assert_eq!(import.import(), 0, "nothing new");
    std::fs::write(&file, "one\ntwo\nthree\nfour\n")?;
    assert_eq!(import.import(), 2, "three and four");

    std::fs::write(&file, "fresh\n")?;
    assert_eq!(
        import.import(),
        1,
        "a shorter journal is read from its start"
    );
    std::fs::remove_dir_all(&root)?;
    Ok(())
}
