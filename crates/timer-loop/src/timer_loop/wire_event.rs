use performance::{Event, Flag, Rejection};
use protocol::{Setting, WireEvent};

/// The form an event takes on the link.
pub(super) fn to_wire_event(event: Event) -> WireEvent {
    match event {
        Event::PerformanceStarted => WireEvent::PerformanceStarted,
        Event::HandOverStarted { from, to } => WireEvent::HandOverStarted { from, to },
        Event::HandOverCompleted { song_id } => WireEvent::HandOverCompleted { song_id },
        Event::HardStopReached { song_id } => WireEvent::HardStopReached { song_id },
        Event::PerformanceFinished => WireEvent::PerformanceFinished,
        Event::FlagChanged { flag, enabled } => WireEvent::SettingChanged {
            setting: match flag {
                Flag::Autoplay => Setting::Autoplay,
                Flag::CountIn => Setting::CountIn,
            },
            enabled,
        },
        Event::SeekPerformed { to } => WireEvent::SeekPerformed { to: to.get() },
        Event::CommandRejected(rejection) => WireEvent::CommandRejected {
            reason: reason(rejection).into(),
        },
    }
}

fn reason(rejection: Rejection) -> &'static str {
    match rejection {
        Rejection::NothingToPlay => "the setlist has no songs",
        Rejection::NoNextSong => "this is the last song",
        Rejection::NoSuchSong => "there is no such song",
        Rejection::NoPreviousSong => "this is the first song",
        Rejection::OutsideSong => "that position is outside the song",
        Rejection::NotNow => "not possible right now",
    }
}
