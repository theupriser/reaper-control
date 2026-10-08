//! Decides which app events a person on stage needs to see, and in which words.

use protocol::{Notice, NoticeLevel};

use crate::app_event::AppEvent;
use crate::link_cause::LinkCause;
use crate::link_health::LinkHealth;

/// The notice for `event`, or `None` when it is only for the log.
#[must_use]
pub fn notice_for_event(event: &AppEvent) -> Option<Notice> {
    match event {
        AppEvent::CommandRefused { error, .. } => Some(notice(
            "command",
            NoticeLevel::Warning,
            format!("Command not sent: {error}"),
        )),
        AppEvent::CommandInvalid { refusal, .. } => Some(notice(
            "command",
            NoticeLevel::Warning,
            format!("Command not sent: {refusal}"),
        )),
        AppEvent::CommandQueueFull(_) => Some(notice(
            "command",
            NoticeLevel::Warning,
            "Command not sent: too many commands are waiting for REAPER".into(),
        )),
        AppEvent::CommandDropped(_) => Some(notice(
            "command",
            NoticeLevel::Info,
            "Repeated command ignored".into(),
        )),
        AppEvent::CommandTimedOut { .. } => Some(notice(
            "command",
            NoticeLevel::Error,
            "REAPER did not answer a command in time".into(),
        )),
        AppEvent::ExtensionRefused { reason, .. } => Some(notice(
            "command",
            NoticeLevel::Warning,
            format!("REAPER refused the command: {reason}"),
        )),
        AppEvent::IntentRefused { refusal, .. } => Some(notice(
            "command",
            NoticeLevel::Warning,
            capitalised(&refusal.to_string()),
        )),
        AppEvent::LinkHealthChanged { health } => Some(health_notice(health)),
        AppEvent::EventsMissed { .. } => Some(notice(
            "events",
            NoticeLevel::Warning,
            "Some performance events were missed while the link was down".into(),
        )),
        _ => None,
    }
}

fn health_notice(health: &LinkHealth) -> Notice {
    let (level, text) = match health {
        LinkHealth::Connected => (NoticeLevel::Info, "Connected to REAPER"),
        LinkHealth::Degraded => (NoticeLevel::Warning, "REAPER is slow to answer"),
        LinkHealth::Lost => (NoticeLevel::Error, "The link to REAPER is lost"),
        LinkHealth::Dead(LinkCause::ReaperNotRunning) => {
            (NoticeLevel::Error, "REAPER is not running")
        }
        LinkHealth::Dead(LinkCause::ExtensionNotLoaded) => (
            NoticeLevel::Error,
            "REAPER is running but the extension is not loaded",
        ),
        LinkHealth::Dead(LinkCause::ExtensionOutdated { .. }) => (
            NoticeLevel::Error,
            "The extension in REAPER is a different version than this app",
        ),
    };
    notice("link", level, text.into())
}

fn notice(key: &str, level: NoticeLevel, text: String) -> Notice {
    Notice {
        key: key.into(),
        level,
        text,
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
