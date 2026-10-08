//! Decides which app events a person on stage needs to see, and in which words.

use protocol::{Notice, NoticeLevel};

use crate::app_event::AppEvent;
use crate::link_cause::LinkCause;
use crate::link_health::LinkHealth;

/// The notice for `event`, or `None` when it is only for the log.
#[must_use]
pub fn notice_for_event(event: &AppEvent) -> Option<Notice> {
    match event {
        AppEvent::CommandRefused { error, .. } => Some(command_notice(
            NoticeLevel::Warning,
            "Command not sent",
            capitalised(&error.to_string()),
        )),
        AppEvent::CommandInvalid { refusal, .. } => Some(command_notice(
            NoticeLevel::Warning,
            "Command not sent",
            capitalised(&refusal.to_string()),
        )),
        AppEvent::CommandQueueFull(_) => Some(command_notice(
            NoticeLevel::Warning,
            "Command not sent",
            "Too many commands are waiting for REAPER.".into(),
        )),
        AppEvent::CommandDropped(_) => Some(command_notice(
            NoticeLevel::Info,
            "Repeat ignored",
            "The same command was just sent.".into(),
        )),
        AppEvent::CommandTimedOut { .. } => Some(command_notice(
            NoticeLevel::Error,
            "REAPER did not answer",
            "A command was not confirmed in time.".into(),
        )),
        AppEvent::ExtensionRefused { reason, .. } => Some(command_notice(
            NoticeLevel::Warning,
            "REAPER refused the command",
            capitalised(reason),
        )),
        AppEvent::IntentRefused { refusal, .. } => Some(command_notice(
            NoticeLevel::Warning,
            "Not possible now",
            capitalised(&refusal.to_string()),
        )),
        AppEvent::LinkHealthChanged { health } => Some(health_notice(health)),
        AppEvent::EventsMissed { .. } => Some(notice(
            "events",
            NoticeLevel::Warning,
            "Events missed",
            "Some performance events were missed while the link was down.",
        )),
        _ => None,
    }
}

/// What is wrong with the link, in a few words for the sidebar; `None` while the link works.
#[must_use]
pub fn link_problem(health: &LinkHealth) -> Option<String> {
    match health {
        LinkHealth::Connected | LinkHealth::Degraded => None,
        _ => Some(health_notice(health).title),
    }
}

fn health_notice(health: &LinkHealth) -> Notice {
    let (level, title, text) = match health {
        LinkHealth::Connected => (NoticeLevel::Info, "Connected to REAPER", "The link works."),
        LinkHealth::Degraded => (
            NoticeLevel::Warning,
            "REAPER is slow to answer",
            "The link is still up.",
        ),
        LinkHealth::Lost => (
            NoticeLevel::Error,
            "Could not reach REAPER",
            "Check that REAPER is open. We keep trying.",
        ),
        LinkHealth::Dead(LinkCause::ReaperNotRunning) => (
            NoticeLevel::Error,
            "REAPER is not running",
            "Open REAPER. We keep trying.",
        ),
        LinkHealth::Dead(LinkCause::ExtensionNotLoaded) => (
            NoticeLevel::Error,
            "The extension is not loaded",
            "REAPER is running but the Reaper Control extension does not answer.",
        ),
        LinkHealth::Dead(LinkCause::ExtensionOutdated { .. }) => (
            NoticeLevel::Error,
            "The extension is another version",
            "Update the extension in REAPER to match this app.",
        ),
        LinkHealth::Dead(LinkCause::ExtensionFaulted { reason }) => {
            return notice(
                "link",
                NoticeLevel::Error,
                "The extension turned itself off",
                &capitalised(reason),
            );
        }
    };
    notice("link", level, title, text)
}

fn command_notice(level: NoticeLevel, title: &str, text: String) -> Notice {
    notice("command", level, title, &text)
}

fn notice(key: &str, level: NoticeLevel, title: &str, text: &str) -> Notice {
    Notice {
        key: key.into(),
        level,
        title: title.into(),
        text: text.into(),
    }
}

fn capitalised(text: &str) -> String {
    let mut letters = text.chars();
    letters
        .next()
        .map(|first| first.to_uppercase().chain(letters).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
