//! Turns what the installer found into the four wizard steps. Pure: the UI only draws the result.

use protocol::{WizardStep, WizardStepId, WizardStepStatus};

use crate::install_item_kind::InstallItemKind;
use crate::install_report::InstallReport;
use crate::install_status::InstallStatus;

/// The steps for `report`, given whether REAPER runs and whether the extension has answered.
#[must_use]
pub fn wizard_step_list(
    report: &InstallReport,
    reaper_running: bool,
    connected: bool,
) -> Vec<WizardStep> {
    let find = find_step(report);
    let install = install_step(report, find.status);
    let restart = restart_step(install.status, reaper_running, connected);
    let connect = WizardStep {
        id: WizardStepId::Connect,
        status: if connected {
            WizardStepStatus::Done
        } else {
            WizardStepStatus::Waiting
        },
        advice: String::new(),
    };
    vec![find, install, restart, connect]
}

fn find_step(report: &InstallReport) -> WizardStep {
    let kinds = [
        InstallItemKind::ResourceFolder,
        InstallItemKind::Writable,
        InstallItemKind::Architecture,
    ];
    let items = report
        .items
        .iter()
        .filter(|item| kinds.contains(&item.kind));
    let blocking = items
        .clone()
        .find(|item| item.status == InstallStatus::Manual);
    let note = items
        .clone()
        .find(|item| item.status == InstallStatus::Unknown);
    let (status, advice) = match (blocking, note) {
        (Some(item), _) => (WizardStepStatus::NeedsYou, item.advice.clone()),
        (None, Some(item)) => (WizardStepStatus::Done, item.advice.clone()),
        (None, None) => (WizardStepStatus::Done, String::new()),
    };
    WizardStep {
        id: WizardStepId::FindReaper,
        status,
        advice,
    }
}

fn install_step(report: &InstallReport, find: WizardStepStatus) -> WizardStep {
    let item = report
        .items
        .iter()
        .find(|item| item.kind == InstallItemKind::Extension);
    let (status, advice) = match (find, item) {
        (WizardStepStatus::Done, Some(item)) => (
            match item.status {
                InstallStatus::Ok => WizardStepStatus::Done,
                InstallStatus::Manual => WizardStepStatus::NeedsYou,
                InstallStatus::Fixable | InstallStatus::Unknown => WizardStepStatus::Current,
            },
            item.advice.clone(),
        ),
        _ => (WizardStepStatus::Waiting, String::new()),
    };
    WizardStep {
        id: WizardStepId::InstallExtension,
        status,
        advice,
    }
}

fn restart_step(install: WizardStepStatus, reaper_running: bool, connected: bool) -> WizardStep {
    let (status, advice) = if connected {
        (WizardStepStatus::Done, "")
    } else if install != WizardStepStatus::Done {
        (WizardStepStatus::Waiting, "")
    } else if reaper_running {
        (
            WizardStepStatus::Current,
            "REAPER loads extensions only when it starts. Quit REAPER and open it again.",
        )
    } else {
        (
            WizardStepStatus::Current,
            "Open REAPER. It loads the extension when it starts.",
        )
    };
    WizardStep {
        id: WizardStepId::RestartReaper,
        status,
        advice: advice.to_owned(),
    }
}
