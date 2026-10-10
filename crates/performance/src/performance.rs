use shared_kernel::Seconds;

use crate::reaper_stop_bridge::ReaperStopBridge;
use crate::{
    Effect, Event, Flag, Flags, HandOverPolicy, Input, Output, Phase, PlannedSong, Rejection,
};

/// How close two song edges must be to count as contiguous.
const CONTIGUOUS: f64 = 0.001;

/// How many ticks in a row REAPER's transport must disagree with the phase before the phase gives
/// way; one stale reading right after a command must not flip it.
const DISAGREEING_TICKS: u8 = 3;

/// How early before the `!1008` marker REAPER may stop and still count as stopped by it.
const MARKER_TOLERANCE: f64 = 0.25;

/// The performance of one setlist: the state machine behind the stage app.
/// Pure: it gets inputs (commands and ticks with an injected clock) and
/// returns effects and events; it never touches REAPER.
#[derive(Debug, Clone)]
pub struct Performance {
    songs: Vec<PlannedSong>,
    current: Option<usize>,
    phase: Phase,
    flags: Flags,
    policy: HandOverPolicy,
    last_hand_over: Option<Seconds>,
    count_in_target: Option<Seconds>,
    disagreeing: u8,
    reaper_playing: bool,
    /// Boxed: it only exists while REAPER stands still, and keeps `Performance` under 128 bytes.
    bridge: Option<Box<ReaperStopBridge>>,
    held: Option<Seconds>,
}

impl Performance {
    /// A performance at the first song, idle.
    pub fn new(songs: Vec<PlannedSong>, flags: Flags, policy: HandOverPolicy) -> Self {
        let current = if songs.is_empty() { None } else { Some(0) };
        Self {
            songs,
            current,
            phase: Phase::Idle,
            flags,
            policy,
            last_hand_over: None,
            count_in_target: None,
            disagreeing: 0,
            reaper_playing: true,
            bridge: None,
            held: None,
        }
    }

    /// Swaps in the songs after an edit and keeps the show going: the phase, settings and last
    /// hand-over stay, and the current song is found again by its identity. Returns false, and
    /// changes nothing, when the current song is gone; the caller then starts over.
    pub fn replan(&mut self, songs: Vec<PlannedSong>) -> bool {
        let Some(current) = self.current() else {
            return false;
        };
        let Some(index) = songs
            .iter()
            .position(|song| song.song_id == current.song_id)
        else {
            return false;
        };
        self.songs = songs;
        self.current = Some(index);
        true
    }

    /// The current phase.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// The playback settings.
    pub fn flags(&self) -> Flags {
        self.flags
    }

    /// Index of the current song in the setlist.
    pub fn current_index(&self) -> Option<usize> {
        self.current
    }

    /// The current song.
    pub fn current(&self) -> Option<&PlannedSong> {
        self.current.and_then(|index| self.songs.get(index))
    }

    /// Applies one input and says what to do.
    pub fn step(&mut self, input: Input) -> Output {
        let mut out = Output::default();
        if self.pause_or_resume_the_run_on(&input) {
            return out;
        }
        if !matches!(input, Input::Observed { .. } | Input::Tick { .. }) {
            self.disagreeing = 0;
            self.bridge = None;
            self.held = None;
        }
        match input {
            Input::Tick { now, position } => self.tick(now, position, &mut out),
            Input::Observed { playing } => self.observe(playing),
            Input::Play => self.play(&mut out),
            Input::Pause => self.pause(&mut out),
            Input::Next => self.step_by(1, &mut out),
            Input::Previous => self.step_by(-1, &mut out),
            Input::RestartSong => self.restart(&mut out),
            Input::GoToSong { index } => self.go_to_song(index, &mut out),
            Input::Seek { position } => self.seek(position, &mut out),
            Input::SeekCue { position } => self.seek_cue(position, &mut out),
            Input::SetFlag { flag, enabled } => self.set_flag(flag, enabled, &mut out),
        }
        out
    }

    /// While the timeline runs on after REAPER stopped at the marker, Pause freezes it and Play
    /// runs it on again. REAPER already stands still, so neither says anything to it. True when
    /// the input was one of those.
    fn pause_or_resume_the_run_on(&mut self, input: &Input) -> bool {
        let Some(bridge) = self.bridge.as_mut() else {
            return false;
        };
        match input {
            Input::Pause if bridge.is_frozen() => {}
            Input::Pause if self.phase == Phase::Playing => {
                bridge.freeze();
                self.phase = Phase::Paused;
            }
            Input::Play if bridge.is_frozen() => self.phase = Phase::Playing,
            Input::Play if self.phase == Phase::Playing => {}
            _ => return false,
        }
        self.disagreeing = 0;
        true
    }

    fn reject(&self, why: Rejection, out: &mut Output) {
        out.events.push(Event::CommandRejected(why));
    }

    fn song_at(&self, index: usize) -> Option<&PlannedSong> {
        self.songs.get(index)
    }

    fn next_song(&self) -> Option<&PlannedSong> {
        self.current.and_then(|index| self.song_at(index + 1))
    }

    fn start_of(&self, index: usize) -> Option<Seconds> {
        self.song_at(index).map(|song| song.window.start())
    }

    fn set_flag(&mut self, flag: Flag, enabled: bool, out: &mut Output) {
        if self.flags.set(flag, enabled) {
            out.events.push(Event::FlagChanged { flag, enabled });
        }
    }

    /// Follows REAPER's real transport once it has disagreed with the phase for a few ticks. No
    /// effect is needed: REAPER already did what the phase now admits.
    fn observe(&mut self, playing: bool) {
        self.reaper_playing = playing;
        if self.bridge.is_some() {
            if playing {
                self.bridge = None;
                self.disagreeing = 0;
            }
            return;
        }
        let expected = match self.phase {
            Phase::Playing | Phase::CountingIn | Phase::HandingOver => Some(true),
            Phase::Paused => Some(false),
            Phase::Idle | Phase::HardStopped | Phase::Finished => None,
        };
        if expected.is_none_or(|expected| expected == playing) {
            self.disagreeing = 0;
            return;
        }
        self.disagreeing += 1;
        if self.disagreeing >= DISAGREEING_TICKS {
            self.disagreeing = 0;
            self.count_in_target = None;
            self.phase = if playing {
                Phase::Playing
            } else {
                Phase::Paused
            };
        }
    }

    fn play(&mut self, out: &mut Output) {
        let Some(current) = self.current else {
            return self.reject(Rejection::NothingToPlay, out);
        };
        match self.phase {
            Phase::Paused => self.resume(out),
            Phase::Idle => self.begin_at(current, out),
            Phase::Finished => self.begin_at(0, out),
            Phase::HardStopped => self.begin_at(current + 1, out),
            Phase::Playing | Phase::CountingIn | Phase::HandingOver => {}
        }
    }

    fn resume(&mut self, out: &mut Output) {
        out.effects.push(Effect::Play);
        self.phase = Phase::Playing;
    }

    /// Starts playing at the start of a song (first play, restart, after a hard stop).
    fn begin_at(&mut self, index: usize, out: &mut Output) {
        let Some(start) = self.start_of(index) else {
            return self.reject(Rejection::NothingToPlay, out);
        };
        let started = matches!(self.phase, Phase::Idle | Phase::Finished);
        self.current = Some(index);
        out.effects.push(Effect::SeekTo(start));
        out.effects.push(Effect::Play);
        out.events.push(if started {
            Event::PerformanceStarted
        } else {
            Event::SeekPerformed { to: start }
        });
        self.phase = Phase::Playing;
    }

    fn pause(&mut self, out: &mut Output) {
        if matches!(
            self.phase,
            Phase::Playing | Phase::CountingIn | Phase::HandingOver
        ) {
            self.cancel_count_in(out);
            out.effects.push(Effect::Pause);
            self.phase = Phase::Paused;
        }
    }

    fn cancel_count_in(&mut self, out: &mut Output) {
        if self.count_in_target.take().is_some() {
            out.effects.push(Effect::SetCountIn(false));
        }
    }

    fn step_by(&mut self, delta: isize, out: &mut Output) {
        let Some(current) = self.current else {
            return self.reject(Rejection::NothingToPlay, out);
        };
        let Some(target) = current.checked_add_signed(delta) else {
            return self.reject(Rejection::NoPreviousSong, out);
        };
        if self.song_at(target).is_none() {
            return self.reject(Rejection::NoNextSong, out);
        }
        self.go_to(target, out);
    }

    fn restart(&mut self, out: &mut Output) {
        match self.current {
            Some(current) => self.go_to(current, out),
            None => self.reject(Rejection::NothingToPlay, out),
        }
    }

    fn go_to_song(&mut self, index: usize, out: &mut Output) {
        if self.song_at(index).is_none() {
            return self.reject(Rejection::NoSuchSong, out);
        }
        self.go_to(index, out);
    }

    /// Manual navigation: honours the setlist, never counts in. Playback continues only when it was
    /// playing and "Auto-resume playback" is on (v1: "only resume if it was already playing").
    fn go_to(&mut self, index: usize, out: &mut Output) {
        let Some(start) = self.start_of(index) else {
            return;
        };
        let was_playing = matches!(
            self.phase,
            Phase::Playing | Phase::CountingIn | Phase::HandingOver
        );
        self.cancel_count_in(out);
        self.current = Some(index);
        out.effects.push(Effect::Pause);
        out.effects.push(Effect::SeekTo(start));
        out.events.push(Event::SeekPerformed { to: start });
        if was_playing && self.flags.autoplay {
            out.effects.push(Effect::Play);
            self.phase = Phase::Playing;
        } else {
            self.phase = Phase::Paused;
        }
    }

    /// Checks that a jump inside the current song is allowed now.
    fn check_jump(&self, position: Seconds, out: &mut Output) -> bool {
        let Some(song) = self.current() else {
            self.reject(Rejection::NothingToPlay, out);
            return false;
        };
        if matches!(self.phase, Phase::HardStopped | Phase::Finished) {
            self.reject(Rejection::NotNow, out);
            return false;
        }
        if !song.window.contains(position) {
            self.reject(Rejection::OutsideSong, out);
            return false;
        }
        true
    }

    fn seek(&mut self, position: Seconds, out: &mut Output) {
        if !self.check_jump(position, out) {
            return;
        }
        self.cancel_count_in(out);
        let playing = matches!(
            self.phase,
            Phase::Playing | Phase::CountingIn | Phase::HandingOver
        );
        if playing {
            out.effects.push(Effect::Pause);
        }
        out.effects.push(Effect::SeekTo(position));
        out.events.push(Event::SeekPerformed { to: position });
        if playing || (self.phase == Phase::Paused && self.flags.autoplay) {
            self.resume(out);
        }
    }

    /// REAPER counts in only when playback starts from a pause, not on a jump while playing, so a
    /// counted-in jump is: pause, move to the cue, arm the count-in, play. REAPER holds the playhead
    /// on the cue while it counts in (measured in REAPER 7.82).
    fn seek_cue(&mut self, position: Seconds, out: &mut Output) {
        if !self.flags.count_in {
            return self.seek(position, out);
        }
        if !self.check_jump(position, out) {
            return;
        }
        if self.phase == Phase::Idle {
            out.events.push(Event::PerformanceStarted);
        }
        out.effects.push(Effect::Pause);
        out.effects.push(Effect::SeekTo(position));
        out.effects.push(Effect::SetCountIn(true));
        out.effects.push(Effect::Play);
        out.events.push(Event::SeekPerformed { to: position });
        self.count_in_target = Some(position);
        self.phase = Phase::CountingIn;
    }

    fn tick(&mut self, now: Seconds, position: Seconds, out: &mut Output) {
        match self.phase {
            Phase::CountingIn => self.finish_count_in(position, out),
            Phase::HandingOver => self.finish_hand_over(position, out),
            Phase::Playing => {
                let position = self.follow_reaper_stop(now, position);
                self.watch_song_end(now, position, out);
            }
            _ => {}
        }
    }

    /// The position the timeline is at: REAPER's, unless REAPER stopped at the `!1008` marker and
    /// the performance has been keeping time since.
    fn follow_reaper_stop(&mut self, now: Seconds, position: Seconds) -> Seconds {
        if let Some(bridge) = self.bridge.as_mut() {
            if bridge.is_frozen() {
                bridge.resume(now);
            }
            bridge.see(now);
            return Seconds::new(bridge.position_at(now)).unwrap_or(position);
        }
        let at_marker = self
            .current()
            .and_then(|song| song.hard_stop_marker)
            .is_some_and(|marker| position.get() >= marker.get() - MARKER_TOLERANCE);
        if !self.reaper_playing && at_marker {
            self.bridge = Some(Box::new(ReaperStopBridge::begin(now, position)));
            self.disagreeing = 0;
        }
        position
    }

    /// Where to show the timeline: the position the performance kept time to while REAPER stood
    /// still, else REAPER's own.
    pub fn shown_position(&self, now: Seconds, reaper: Seconds) -> Seconds {
        if let Some(bridge) = &self.bridge {
            let end = self.current().map(|song| song.window.end().get());
            let position = end.map_or(bridge.position_at(now), |end| {
                bridge.position_at(now).min(end)
            });
            return Seconds::new(position).unwrap_or(reaper);
        }
        self.held.unwrap_or(reaper)
    }

    fn finish_count_in(&mut self, position: Seconds, out: &mut Output) {
        let reached = self
            .count_in_target
            .is_none_or(|target| position.get() > target.get());
        if reached {
            self.cancel_count_in(out);
            self.phase = Phase::Playing;
        }
    }

    fn finish_hand_over(&mut self, position: Seconds, out: &mut Output) {
        let Some(song) = self.current() else {
            return;
        };
        if song.window.contains(position) {
            let song_id = song.song_id.clone();
            out.events.push(Event::HandOverCompleted { song_id });
            self.phase = Phase::Playing;
        }
    }

    fn watch_song_end(&mut self, now: Seconds, position: Seconds, out: &mut Output) {
        let Some(song) = self.current() else {
            return;
        };
        if !self.policy.is_due(position, &song.window) {
            return;
        }
        if song.hard_stop {
            return self.hard_stop(out);
        }
        if self.policy.is_debounced(now, self.last_hand_over) {
            return;
        }
        match self.current.map(|index| index + 1) {
            Some(next) if self.song_at(next).is_some() => self.hand_over(now, next, out),
            _ => self.finish(out),
        }
    }

    fn hard_stop(&mut self, out: &mut Output) {
        let Some(song) = self.current() else {
            return;
        };
        out.events.push(Event::HardStopReached {
            song_id: song.song_id.clone(),
        });
        out.effects.push(Effect::Pause);
        self.held = self
            .bridge
            .take()
            .and_then(|_| self.current().map(|song| song.window.end()));
        if self.next_song().is_some() {
            self.phase = Phase::HardStopped;
        } else {
            self.finish_events(out);
        }
    }

    fn finish(&mut self, out: &mut Output) {
        out.effects.push(Effect::Pause);
        self.finish_events(out);
    }

    fn finish_events(&mut self, out: &mut Output) {
        out.events.push(Event::PerformanceFinished);
        self.phase = Phase::Finished;
    }

    fn hand_over(&mut self, now: Seconds, next: usize, out: &mut Output) {
        let (Some(from), Some(to)) = (self.current(), self.song_at(next)) else {
            return;
        };
        let contiguous = (to.window.start().get() - from.window.end().get()).abs() < CONTIGUOUS;
        out.events.push(Event::HandOverStarted {
            from: from.song_id.clone(),
            to: to.song_id.clone(),
        });
        if !contiguous {
            out.effects.push(Effect::SeekTo(to.window.start()));
        }
        self.last_hand_over = Some(now);
        self.current = Some(next);
        self.phase = Phase::HandingOver;
    }
}
