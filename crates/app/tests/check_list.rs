//! The pre-show checklist: every check fails on its own and the rest stay as they were (WP 6.7).

use app::{AppConfig, check_list};
use protocol::{
    Catalog, CheckId, CheckStatus, ChecklistView, EntryInfo, InstallationView, LinkStatus,
    LinkView, SetlistInfo, Settings, SongInfo, WizardStep, WizardStepId, WizardStepStatus,
};

fn song(id: &str) -> SongInfo {
    SongInfo {
        id: id.into(),
        number: 1,
        name: id.into(),
        start: 0.0,
        end: 10.0,
        colour: None,
        hard_stop: false,
        length: None,
        bpm: None,
    }
}

fn setlist(entries: &[&str]) -> SetlistInfo {
    SetlistInfo {
        id: "set".into(),
        name: "Friday".into(),
        revision: 1,
        entries: entries
            .iter()
            .zip(1..)
            .map(|(song_id, id)| EntryInfo {
                id,
                song_id: (*song_id).into(),
            })
            .collect(),
    }
}

fn link(songs: &[&str], setlist: Option<SetlistInfo>) -> LinkView {
    LinkView {
        status: LinkStatus::Connected {
            extension_version: "1.0".into(),
        },
        live: None,
        catalog: Box::new(Catalog {
            project_songs: songs.iter().map(|id| song(id)).collect(),
            active_setlist: setlist.as_ref().map(|found| found.id.clone()),
            setlists: setlist.into_iter().collect(),
            ..Catalog::default()
        }),
    }
}

fn installed(status: WizardStepStatus) -> InstallationView {
    InstallationView {
        steps: vec![WizardStep {
            id: WizardStepId::InstallExtension,
            status,
            advice: String::new(),
        }],
        folder: String::new(),
        can_install: false,
        complete: true,
    }
}

fn settings(midi_enabled: bool, device: Option<&str>) -> Box<Settings> {
    let mut found = AppConfig::default().view(Vec::new()).settings;
    found.midi_enabled = midi_enabled;
    found.midi_device_name = device.map(Into::into);
    found
}

fn statuses(view: &ChecklistView) -> Vec<(CheckId, CheckStatus)> {
    view.items
        .iter()
        .map(|item| (item.id, item.status))
        .collect()
}

fn status_of(view: &ChecklistView, id: CheckId) -> Option<CheckStatus> {
    view.items
        .iter()
        .find(|item| item.id == id)
        .map(|item| item.status)
}

use CheckId::{Connected, ExtensionCurrent, MidiPresent, SetlistValid, SongsFound};
use CheckStatus::{Failed, Passed, Skipped};

#[test]
fn everything_in_order_is_ready() {
    let view = check_list(
        &link(&["a", "b"], Some(setlist(&["a", "b"]))),
        &settings(true, Some("Pedal")),
        &["Pedal".into()],
        Some(&installed(WizardStepStatus::Done)),
    );
    assert_eq!(
        statuses(&view),
        [
            (ExtensionCurrent, Passed),
            (Connected, Passed),
            (SongsFound, Passed),
            (SetlistValid, Passed),
            (MidiPresent, Passed)
        ]
    );
    assert!(view.ready);
}

#[test]
fn an_extension_that_is_not_the_bundled_one_fails_with_the_installers_advice() {
    let mut stale = installed(WizardStepStatus::Current);
    stale.steps[0].advice = "Update it.".into();
    let view = check_list(
        &link(&["a"], None),
        &settings(false, None),
        &[],
        Some(&stale),
    );
    assert_eq!(status_of(&view, ExtensionCurrent), Some(Failed));
    assert_eq!(view.items[0].detail, "Update it.");
    assert!(!view.ready);
}

#[test]
fn a_system_without_an_installer_skips_the_extension_check() {
    let view = check_list(&link(&["a"], None), &settings(false, None), &[], None);
    assert_eq!(status_of(&view, ExtensionCurrent), Some(Skipped));
    assert!(view.ready);
}

#[test]
fn no_connection_fails_and_the_checks_that_need_it_are_skipped() {
    let view = check_list(
        &LinkView::default(),
        &settings(false, None),
        &[],
        Some(&installed(WizardStepStatus::Done)),
    );
    assert_eq!(status_of(&view, Connected), Some(Failed));
    assert_eq!(status_of(&view, SongsFound), Some(Skipped));
    assert_eq!(status_of(&view, SetlistValid), Some(Skipped));
    assert!(!view.ready);
}

#[test]
fn a_project_without_regions_fails() {
    let view = check_list(&link(&[], None), &settings(false, None), &[], None);
    assert_eq!(status_of(&view, SongsFound), Some(Failed));
}

#[test]
fn a_setlist_with_a_song_that_is_gone_fails_and_names_how_many() {
    let view = check_list(
        &link(&["a"], Some(setlist(&["a", "gone", "gone-too"]))),
        &settings(false, None),
        &[],
        None,
    );
    assert_eq!(status_of(&view, SetlistValid), Some(Failed));
    assert!(view.items[3].detail.contains("2 entries"));
}

#[test]
fn an_empty_played_setlist_fails_and_no_setlist_passes() {
    let empty = check_list(
        &link(&["a"], Some(setlist(&[]))),
        &settings(false, None),
        &[],
        None,
    );
    assert_eq!(status_of(&empty, SetlistValid), Some(Failed));
    let none = check_list(&link(&["a"], None), &settings(false, None), &[], None);
    assert_eq!(status_of(&none, SetlistValid), Some(Passed));
}

#[test]
fn midi_is_skipped_when_off_and_fails_when_the_chosen_device_is_missing() {
    let off = check_list(&link(&["a"], None), &settings(false, None), &[], None);
    assert_eq!(status_of(&off, MidiPresent), Some(Skipped));
    let missing = check_list(
        &link(&["a"], None),
        &settings(true, Some("Pedal")),
        &["Other".into()],
        None,
    );
    assert_eq!(status_of(&missing, MidiPresent), Some(Failed));
    let none_found = check_list(&link(&["a"], None), &settings(true, None), &[], None);
    assert_eq!(status_of(&none_found, MidiPresent), Some(Failed));
    let any = check_list(
        &link(&["a"], None),
        &settings(true, None),
        &["Other".into()],
        None,
    );
    assert_eq!(status_of(&any, MidiPresent), Some(Passed));
}
