use protocol::{Command, NoticeLevel};

use super::notice_for_event;
use crate::app_event::AppEvent;
use crate::intent::Intent;
use crate::intent_refusal::IntentRefusal;
use crate::link_cause::LinkCause;
use crate::link_health::LinkHealth;

#[test]
fn an_unanswered_command_is_an_error_a_person_sees() {
    let notice = notice_for_event(&AppEvent::CommandTimedOut {
        id: 3,
        command: Command::Next,
    });
    assert_eq!(notice.as_ref().map(|n| n.level), Some(NoticeLevel::Error));
    assert_eq!(notice.map(|n| n.key), Some("command".to_owned()));
}

#[test]
fn a_refused_intent_is_said_in_a_sentence() {
    let notice = notice_for_event(&AppEvent::IntentRefused {
        intent: Intent::Next,
        refusal: IntentRefusal::NoNextSong,
    });
    assert_eq!(
        notice.map(|notice| notice.text),
        Some("There is no next song".to_owned())
    );
}

#[test]
fn health_changes_share_one_key_so_the_newest_wins() {
    let dead = notice_for_event(&AppEvent::LinkHealthChanged {
        health: LinkHealth::Dead(LinkCause::ReaperNotRunning),
    });
    let back = notice_for_event(&AppEvent::LinkHealthChanged {
        health: LinkHealth::Connected,
    });
    assert_eq!(
        dead.as_ref().map(|n| n.title.as_str()),
        Some("REAPER is not running")
    );
    assert_eq!(dead.map(|n| n.key), back.map(|n| n.key));
}

#[test]
fn routine_events_stay_in_the_log() {
    assert_eq!(
        notice_for_event(&AppEvent::CommandSent(Command::Play)),
        None
    );
    assert_eq!(
        notice_for_event(&AppEvent::CommandAcknowledged { id: 1 }),
        None
    );
}
