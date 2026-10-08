mod plan_songs;

use performance::{Effect, Flags, HandOverPolicy, Input, Performance, Phase, PlannedSong};
use reaper_port::ReaperPort;

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
}

impl<Port: ReaperPort> TimerLoop<Port> {
    /// Reads the project once and starts idle at the first song.
    pub fn new(port: Port) -> Self {
        let songs = plan_songs(&port.regions(), &port.markers());
        let seen_change_count = port.change_count();
        let performance = Self::fresh(&port, songs.clone());
        Self {
            port,
            performance,
            songs,
            seen_change_count,
            rebuilds: 0,
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
        let songs = plan_songs(&self.port.regions(), &self.port.markers());
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

#[cfg(test)]
mod tests;
