//! The pre-show checklist (SPEC S-7). Pure: the shell gathers what the checks look at, the UI draws the result.

use protocol::{
    CheckId, CheckItem, CheckStatus, ChecklistView, InstallationView, LinkStatus, LinkView,
    Settings, SongInfo, WizardStepId, WizardStepStatus,
};

/// The checks for the state of the link, the saved settings, the MIDI devices found now and, when
/// this system has an installer, what the installer found.
#[must_use]
pub fn check_list(
    link: &LinkView,
    settings: &Settings,
    devices: &[String],
    installation: Option<&InstallationView>,
) -> ChecklistView {
    let connected = matches!(link.status, LinkStatus::Connected { .. });
    let items = vec![
        extension_current(installation),
        connection(&link.status),
        songs_found(connected, &link.catalog.project_songs),
        setlist_valid(connected, link),
        midi_present(settings, devices),
    ];
    let ready = items.iter().all(|item| item.status != CheckStatus::Failed);
    ChecklistView { items, ready }
}

fn item(id: CheckId, status: CheckStatus, detail: impl Into<String>) -> CheckItem {
    CheckItem {
        id,
        status,
        detail: detail.into(),
    }
}

fn waiting(id: CheckId) -> CheckItem {
    item(id, CheckStatus::Skipped, "Waiting for the connection.")
}

fn extension_current(installation: Option<&InstallationView>) -> CheckItem {
    let step = installation.and_then(|view| {
        view.steps
            .iter()
            .find(|step| step.id == WizardStepId::InstallExtension)
    });
    match step {
        None => item(
            CheckId::ExtensionCurrent,
            CheckStatus::Skipped,
            "This system has no installer.",
        ),
        Some(step) if step.status == WizardStepStatus::Done => item(
            CheckId::ExtensionCurrent,
            CheckStatus::Passed,
            "The installed extension is the one that ships with this app.",
        ),
        Some(step) if step.advice.is_empty() => item(
            CheckId::ExtensionCurrent,
            CheckStatus::Failed,
            "Install the extension from Settings.",
        ),
        Some(step) => item(
            CheckId::ExtensionCurrent,
            CheckStatus::Failed,
            step.advice.clone(),
        ),
    }
}

fn connection(status: &LinkStatus) -> CheckItem {
    match status {
        LinkStatus::Connected { extension_version } => item(
            CheckId::Connected,
            CheckStatus::Passed,
            format!("Connected to extension {extension_version}."),
        ),
        LinkStatus::NotRunning => item(
            CheckId::Connected,
            CheckStatus::Failed,
            "REAPER is closed or the extension did not load. Open REAPER.",
        ),
    }
}

fn songs_found(connected: bool, songs: &[SongInfo]) -> CheckItem {
    if !connected {
        return waiting(CheckId::SongsFound);
    }
    match songs.len() {
        0 => item(
            CheckId::SongsFound,
            CheckStatus::Failed,
            "The project has no regions. Add one region per song in REAPER.",
        ),
        1 => item(CheckId::SongsFound, CheckStatus::Passed, "1 song found."),
        count => item(
            CheckId::SongsFound,
            CheckStatus::Passed,
            format!("{count} songs found."),
        ),
    }
}

fn setlist_valid(connected: bool, link: &LinkView) -> CheckItem {
    if !connected {
        return waiting(CheckId::SetlistValid);
    }
    let catalog = &link.catalog;
    let Some(active) = catalog.active_setlist.as_ref() else {
        return item(
            CheckId::SetlistValid,
            CheckStatus::Passed,
            "No setlist is played: the songs play in project order.",
        );
    };
    let Some(setlist) = catalog
        .setlists
        .iter()
        .find(|setlist| &setlist.id == active)
    else {
        return item(
            CheckId::SetlistValid,
            CheckStatus::Failed,
            "The setlist that should play is not in the project. Choose one on Setlists.",
        );
    };
    let missing = setlist
        .entries
        .iter()
        .filter(|entry| {
            !catalog
                .project_songs
                .iter()
                .any(|song| song.id == entry.song_id)
        })
        .count();
    if setlist.entries.is_empty() {
        item(
            CheckId::SetlistValid,
            CheckStatus::Failed,
            format!(
                "Setlist \"{}\" is empty. Add songs on Setlists.",
                setlist.name
            ),
        )
    } else if missing > 0 {
        item(
            CheckId::SetlistValid,
            CheckStatus::Failed,
            format!(
                "Setlist \"{}\" has {missing} entries whose song is gone from the project. Fix them on Setlists.",
                setlist.name
            ),
        )
    } else {
        item(
            CheckId::SetlistValid,
            CheckStatus::Passed,
            format!("Setlist \"{}\" is in order.", setlist.name),
        )
    }
}

fn midi_present(settings: &Settings, devices: &[String]) -> CheckItem {
    if !settings.midi_enabled {
        return item(
            CheckId::MidiPresent,
            CheckStatus::Skipped,
            "MIDI input is off in Settings.",
        );
    }
    match settings.midi_device_name.as_deref() {
        Some(name) if !devices.iter().any(|device| device == name) => item(
            CheckId::MidiPresent,
            CheckStatus::Failed,
            format!(
                "The MIDI device \"{name}\" is not connected. Plug it in or choose another in Settings."
            ),
        ),
        Some(name) => item(
            CheckId::MidiPresent,
            CheckStatus::Passed,
            format!("{name} is connected."),
        ),
        None if devices.is_empty() => item(
            CheckId::MidiPresent,
            CheckStatus::Failed,
            "No MIDI device found. Plug in your controller.",
        ),
        None => item(
            CheckId::MidiPresent,
            CheckStatus::Passed,
            "A MIDI device is connected.",
        ),
    }
}
