use shared_kernel::Seconds;

use crate::{
    Effect, Event, Flag, Flags, HandOverPolicy, Input, Output, Phase, PlannedSong, Rejection,
};

/// How close two song edges must be to count as contiguous.
const CONTIGUOUS: f64 = 0.001;

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
        }
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
        match input {
            Input::Tick { now, position } => self.tick(now, position, &mut out),
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
            Phase::Playing => self.watch_song_end(now, position, out),
            _ => {}
        }
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
