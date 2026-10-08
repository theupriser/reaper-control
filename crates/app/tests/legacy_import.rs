//! Reading v1 setlist files, mapping them onto songs, and the restore-only mirror.

use app::legacy_file::LegacyFile;
use app::legacy_resolver::resolve;
use app::mirror_error::MirrorError;
use app::setlist_mirror::SetlistMirror;
use protocol::SongInfo;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn song(id: &str, name: &str) -> SongInfo {
    SongInfo {
        id: id.into(),
        name: name.into(),
        start: 0.0,
        end: 1.0,
        colour: None,
        hard_stop: false,
        length: None,
        bpm: None,
    }
}

const WITH_METADATA: &str = r#"{
  "setlists": [
    {"id":"s1","name":"Friday","projectId":"project-1-abc","items":[
      {"id":"i1","regionId":"3","name":"Opener","position":0},
      {"id":"i2","regionId":"5","name":"Ballad","position":1},
      {"id":"i3","regionId":"3","name":"Opener","position":2},
      {"id":"i4","regionId":"9","name":"Gone","position":3}]}
  ],
  "metadata": {"selectedSetlistId":"s1","lastUpdated":"2025-01-01T00:00:00.000Z"}
}"#;

const BARE_ARRAY: &str = r#"[{"id":"s2","name":"Old","projectId":"p","items":[{"id":"i","regionId":"1","name":"A","position":0}]}]"#;

#[test]
fn both_v1_layouts_are_read() -> TestResult {
    let file = LegacyFile::parse(WITH_METADATA)?;
    assert_eq!(file.selected.as_deref(), Some("s1"));
    assert_eq!(file.setlists.len(), 1);
    assert_eq!(file.setlists.first().map(|s| s.items.len()), Some(4));

    let bare = LegacyFile::parse(BARE_ARRAY)?;
    assert_eq!(bare.selected, None);
    assert_eq!(bare.setlists.len(), 1);
    Ok(())
}

#[test]
fn files_that_are_not_setlists_are_refused() {
    assert!(LegacyFile::parse("42").is_err());
    assert!(LegacyFile::parse("not json").is_err());
    assert!(LegacyFile::parse(r#"{"setlists":"x"}"#).is_err());
}

#[test]
fn items_map_to_songs_by_name_and_a_missing_song_is_reported() -> TestResult {
    let legacy = LegacyFile::parse(WITH_METADATA)?.setlists;
    let songs = [
        song("{A}", "Opener"),
        song("{B}", "Ballad"),
        song("{C}", "Opener"),
    ];
    let resolved = resolve(legacy.first().ok_or("no setlist")?, &songs);
    let ids: Vec<_> = resolved
        .setlist
        .entries
        .iter()
        .map(|e| e.song_id.as_str())
        .collect();
    assert_eq!(ids, ["{A}", "{B}", "{C}"]);
    let entry_ids: Vec<_> = resolved.setlist.entries.iter().map(|e| e.id).collect();
    assert_eq!(entry_ids, [0, 1, 2]);
    assert_eq!(resolved.unresolved.len(), 1);
    assert_eq!(
        resolved.unresolved.first().map(|i| i.name.as_str()),
        Some("Gone")
    );
    assert_eq!(resolved.setlist.revision, 0);
    println!("{resolved:#?}");
    Ok(())
}

fn mirror(name: &str) -> Result<(SetlistMirror, std::path::PathBuf), std::io::Error> {
    let directory = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    Ok((SetlistMirror::new(directory.join("mirror")), directory))
}

#[test]
fn the_mirror_restores_what_was_saved_and_leaves_no_temporary_file() -> TestResult {
    let (mirror, directory) = mirror("mirror-roundtrip")?;
    assert!(mirror.restore("project-1")?.is_empty());
    let legacy = LegacyFile::parse(WITH_METADATA)?.setlists;
    let setlist = resolve(
        legacy.first().ok_or("no setlist")?,
        &[song("{A}", "Opener")],
    )
    .setlist;
    mirror.save("project-1", std::slice::from_ref(&setlist))?;
    assert_eq!(mirror.restore("project-1")?, vec![setlist]);
    assert!(mirror.restore("project-2")?.is_empty());
    assert_eq!(std::fs::read_dir(directory.join("mirror"))?.count(), 1);
    std::fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn a_damaged_mirror_is_reported_and_unsafe_project_ids_are_refused() -> TestResult {
    let (mirror, directory) = mirror("mirror-bad")?;
    mirror.save("project-1", &[])?;
    std::fs::write(directory.join("mirror/project-1.json"), "{broken")?;
    assert!(matches!(
        mirror.restore("project-1"),
        Err(MirrorError::Parse(_))
    ));
    for bad in ["", "../escape", "a/b", "a.b"] {
        assert!(
            matches!(mirror.save(bad, &[]), Err(MirrorError::BadProjectId(_))),
            "{bad:?}"
        );
    }
    std::fs::remove_dir_all(directory)?;
    Ok(())
}
