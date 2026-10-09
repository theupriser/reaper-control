//! The backup copy follows the project, and a restore or an import ends in `SaveSetlist`.

use std::path::PathBuf;
use std::sync::Arc;

use app::CommandBus;
use app::EventBus;
use app::FakeClock;
use app::FakeDriver;
use app::MirrorKeeper;
use app::MirrorRepository;
use app::QueueSettings;
use app::SetlistMirror;
use app::SetlistTransfer;
use protocol::{Catalog, Command, EntryInfo, LinkView, SetlistInfo, SongInfo};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const V1_FILE: &str = r#"{"setlists":[
  {"id":"v1-friday","name":"Friday","projectId":"project-1","items":[
    {"id":"a","regionId":"1","name":"Opener","position":0},
    {"id":"b","regionId":"2","name":"Ballad","position":1},
    {"id":"c","regionId":"9","name":"Gone","position":2}]},
  {"id":"already","name":"Known","projectId":"project-1","items":[]}
]}"#;

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

fn setlist(id: &str, name: &str, songs: &[&str]) -> SetlistInfo {
    SetlistInfo {
        id: id.into(),
        name: name.into(),
        revision: 1,
        entries: songs
            .iter()
            .enumerate()
            .map(|(index, song_id)| EntryInfo {
                id: index as u64,
                song_id: (*song_id).into(),
            })
            .collect(),
    }
}

fn view(project_id: &str, setlists: Vec<SetlistInfo>) -> LinkView {
    LinkView {
        catalog: Box::new(Catalog {
            project_id: project_id.into(),
            project_songs: vec![song("{A}", "Opener"), song("{B}", "Ballad")],
            setlists,
            ..Catalog::default()
        }),
        ..LinkView::default()
    }
}

struct Rig {
    directory: PathBuf,
    mirror: SetlistMirror,
    driver: Arc<FakeDriver>,
    transfer: SetlistTransfer,
}

fn rig(name: &str, with_v1_file: bool) -> Result<Rig, std::io::Error> {
    let directory =
        std::env::temp_dir().join(format!("rc2-transfer-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let legacy = directory.join("legacy");
    std::fs::create_dir_all(&legacy)?;
    if with_v1_file {
        std::fs::write(legacy.join("project-1.json"), V1_FILE)?;
    }
    let driver = Arc::new(FakeDriver::default());
    let bus = Arc::new(CommandBus::new(
        driver.clone(),
        Arc::new(EventBus::default()),
        Arc::new(FakeClock::default()),
        QueueSettings::default(),
    ));
    let mirror = SetlistMirror::new(directory.join("mirror"));
    let transfer = SetlistTransfer::new(Arc::new(mirror.clone()), Some(legacy), bus);
    Ok(Rig {
        directory,
        mirror,
        driver,
        transfer,
    })
}

#[test]
fn the_keeper_writes_changes_but_never_an_empty_list_over_the_copy() -> TestResult {
    let rig = rig("keeper", false)?;
    let keeper = MirrorKeeper::new(Arc::new(rig.mirror.clone()));
    keeper.observe(&view("", vec![setlist("s", "Friday", &["{A}"])]));
    assert!(rig.mirror.restore("project-1")?.is_empty());
    let friday = setlist("s", "Friday", &["{A}"]);
    keeper.observe(&view("project-1", vec![friday.clone()]));
    assert_eq!(rig.mirror.restore("project-1")?, vec![friday]);
    let longer = setlist("s", "Friday", &["{A}", "{B}"]);
    keeper.observe(&view("project-1", vec![longer.clone()]));
    assert_eq!(rig.mirror.restore("project-1")?, vec![longer.clone()]);
    keeper.observe(&view("project-1", Vec::new()));
    assert_eq!(rig.mirror.restore("project-1")?, vec![longer]);
    std::fs::remove_dir_all(rig.directory)?;
    Ok(())
}

#[test]
fn a_restore_puts_back_only_the_setlists_the_project_lacks() -> TestResult {
    let rig = rig("restore", false)?;
    let kept = setlist("kept", "Kept", &["{A}"]);
    let lost = setlist("lost", "Lost", &["{B}", "{A}"]);
    rig.mirror
        .save("project-1", &[kept.clone(), lost.clone()])?;
    let shown = view("project-1", vec![kept]);
    assert_eq!(
        rig.transfer.view(&shown).restorable,
        vec!["Lost".to_owned()]
    );
    assert_eq!(rig.transfer.restore(&shown)?, 1);
    assert_eq!(
        rig.driver.sent(),
        vec![Command::SaveSetlist {
            id: "lost".into(),
            name: "Lost".into(),
            entries: lost.entries,
            expected_revision: 0,
        }]
    );
    std::fs::remove_dir_all(rig.directory)?;
    Ok(())
}

#[test]
fn v1_setlists_are_offered_with_what_is_missing_and_imported_by_choice() -> TestResult {
    let rig = rig("import", true)?;
    let shown = view("project-1", vec![setlist("already", "Known", &[])]);
    let offered = rig.transfer.view(&shown);
    assert_eq!(offered.problem, None);
    assert_eq!(offered.imports.len(), 1);
    let offer = offered.imports.first().ok_or("no offer")?;
    assert_eq!((offer.id.as_str(), offer.found), ("v1-friday", 2));
    assert_eq!(offer.missing, vec!["Gone".to_owned()]);

    assert_eq!(rig.transfer.import(&shown, &[])?, 0);
    assert!(rig.driver.sent().is_empty());
    assert_eq!(
        rig.transfer
            .import(&shown, &["v1-friday".into(), "already".into()])?,
        1
    );
    let sent = rig.driver.sent();
    let Some(Command::SaveSetlist {
        id,
        name,
        entries,
        expected_revision,
    }) = sent.first()
    else {
        return Err("no SaveSetlist sent".into());
    };
    assert_eq!(
        (id.as_str(), name.as_str(), *expected_revision),
        ("v1-friday", "Friday", 0)
    );
    let songs: Vec<_> = entries.iter().map(|e| e.song_id.as_str()).collect();
    assert_eq!(songs, ["{A}", "{B}"]);
    assert_eq!(sent.len(), 1);
    std::fs::remove_dir_all(rig.directory)?;
    Ok(())
}

#[test]
fn nothing_is_offered_without_a_project_and_a_damaged_file_is_named() -> TestResult {
    let rig = rig("problems", true)?;
    let unknown = rig.transfer.view(&view("", Vec::new()));
    assert!(unknown.problem.is_some());
    assert!(unknown.imports.is_empty());

    std::fs::write(rig.directory.join("legacy/project-1.json"), "{broken")?;
    let shown = view("project-1", Vec::new());
    let damaged = rig.transfer.view(&shown);
    assert!(damaged.problem.is_some_and(|text| text.contains("v1 file")));
    assert!(rig.transfer.import(&shown, &["x".into()]).is_err());

    let nothing = view("project-2", Vec::new());
    assert_eq!(
        rig.transfer.view(&nothing),
        protocol::SetlistTransferView::default()
    );
    std::fs::remove_dir_all(rig.directory)?;
    Ok(())
}
