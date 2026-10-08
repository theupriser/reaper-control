mod build_catalog;
mod link_command;
mod plan_songs;

use performance::{Effect, Flags, HandOverPolicy, Input, Performance, Phase, PlannedSong};
use protocol::message::Outcome;
use protocol::{AppState, Catalog, Command};
use reaper_port::ReaperPort;

use build_catalog::build_catalog;
use link_command::to_input;
use plan_songs::plan_songs;

/// One step of REAPER's timer: read the clock and the play position, step the performance, carry
/// out what it asks for. Generic over the port, so the same code runs against REAPER and against
/// `FakeReaper`. The project is only re-read when its change count moves.
#[derive(Debug)]
pub struct TimerLoop<Port: ReaperPort> {
    port: Port,
    performance: Performance,
    songs: Vec<PlannedSong>,
    seen_change_count: u64,
    rebuilds: u64,
    catalog: Catalog,
}

impl<Port: ReaperPort> TimerLoop<Port> {
    /// Reads the project once and starts idle at the first song.
    pub fn new(port: Port) -> Self {
        let songs = plan_songs(&port.regions(), &port.markers());
        let seen_change_count = port.change_count();
        let performance = Self::fresh(&port, songs.clone());
        let catalog = build_catalog(1, &songs, &port.regions(), &port.markers());
        Self {
            port,
            performance,
            songs,
            seen_change_count,
            rebuilds: 0,
            catalog,
        }
    }

    /// Runs one timer tick.
    pub fn tick(&mut self) {
        self.refresh_if_changed();
        let input = Input::Tick {
            now: self.port.now(),
            position: self.port.position(),
        };
        self.apply(input);
    }

    /// Carries out a command, through the same path as a tick.
    pub fn command(&mut self, input: Input) {
        self.refresh_if_changed();
        self.apply(input);
    }

    /// Carries out a command from the link, or says why it cannot.
    pub fn link_command(&mut self, command: Command) -> Outcome {
        let start = self.song_start();
        match to_input(command, start, self.performance.flags()) {
            Ok(input) => {
                self.command(input);
                Outcome::Done
            }
            Err(reason) => Outcome::Rejected {
                reason: reason.into(),
            },
        }
    }

    /// What the app shows: the phase, the position in the current song and the settings.
    pub fn app_state(&self) -> AppState {
        let flags = self.performance.flags();
        AppState {
            phase: map_phase(self.performance.phase()),
            position: (self.port.position().get() - self.song_start().get()).max(0.0),
            auto_resume: flags.autoplay,
            count_in_on_marker: flags.count_in,
            record_armed: false,
            current_song: self
                .performance
                .current_index()
                .and_then(|index| u32::try_from(index).ok()),
        }
    }

    /// The songs and cues of the project; the revision rises whenever they change.
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// Where the performance is.
    pub fn phase(&self) -> Phase {
        self.performance.phase()
    }

    /// How many times an edit in the project made the performance start over.
    pub fn rebuilds(&self) -> u64 {
        self.rebuilds
    }

    /// The port, for the test probe.
    #[cfg(any(test, feature = "probe"))]
    pub fn port_mut(&mut self) -> &mut Port {
        &mut self.port
    }

    fn song_start(&self) -> shared_kernel::Seconds {
        self.performance
            .current()
            .map_or(shared_kernel::Seconds::ZERO, |song| song.window.start())
    }

    fn fresh(port: &Port, songs: Vec<PlannedSong>) -> Performance {
        let flags = Flags {
            autoplay: false,
            count_in: port.count_in(),
        };
        Performance::new(songs, flags, HandOverPolicy::default())
    }

    /// Re-reads regions and markers after an edit. The performance only starts over when the
    /// songs themselves changed, so a note or an unrelated edit mid-song does not stop the show.
    fn refresh_if_changed(&mut self) {
        let change_count = self.port.change_count();
        if change_count == self.seen_change_count {
            return;
        }
        self.seen_change_count = change_count;
        let regions = self.port.regions();
        let markers = self.port.markers();
        let songs = plan_songs(&regions, &markers);
        let mut catalog = build_catalog(self.catalog.revision, &songs, &regions, &markers);
        if catalog != self.catalog {
            catalog.revision += 1;
            self.catalog = catalog;
        }
        if songs != self.songs {
            self.performance = Self::fresh(&self.port, songs.clone());
            self.songs = songs;
            self.rebuilds += 1;
        }
    }

    fn apply(&mut self, input: Input) {
        let output = self.performance.step(input);
        for effect in output.effects {
            match effect {
                Effect::Play => self.port.play(),
                Effect::Pause => self.port.pause(),
                Effect::SeekTo(position) => self.port.seek(position),
                Effect::SetCountIn(enabled) => self.port.set_count_in(enabled),
            }
        }
    }
}

fn map_phase(phase: Phase) -> protocol::Phase {
    match phase {
        Phase::Idle => protocol::Phase::Idle,
        Phase::Playing => protocol::Phase::Playing,
        Phase::Paused => protocol::Phase::Paused,
        Phase::CountingIn => protocol::Phase::CountingIn,
        Phase::HardStopped => protocol::Phase::HardStopped,
        Phase::HandingOver => protocol::Phase::HandingOver,
        Phase::Finished => protocol::Phase::Finished,
    }
}

#[cfg(test)]
mod tests;
