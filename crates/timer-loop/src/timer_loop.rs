mod build_catalog;
mod link_command;
mod plan_songs;
mod project_identity;
mod project_setlists;
mod setlist_edit;
mod wire_event;

use performance::{Effect, Flags, HandOverPolicy, Input, Performance, Phase, PlannedSong};
use protocol::message::Outcome;
use protocol::{Catalog, Command, Live, Transport, WireEvent};
use reaper_port::{Marker, ReaperPort, Region};

use build_catalog::build_catalog;
use link_command::to_input;
use plan_songs::plan_songs;
use project_setlists::ProjectSetlists;
use setlist_edit::SetlistEdit;
use wire_event::to_wire_event;

/// Events nobody has collected yet; the oldest go first when a link never collects them.
const PENDING_EVENTS: usize = 1024;

/// One step of REAPER's timer: read the clock and the play position, step the performance, carry
/// out what it asks for. Generic over the port, so the same code runs against REAPER and against
/// `FakeReaper`. The project is only re-read when its change count moves.
#[derive(Debug)]
pub struct TimerLoop<Port: ReaperPort> {
    port: Port,
    performance: Performance,
    songs: Vec<PlannedSong>,
    project_setlists: ProjectSetlists,
    seen_change_count: u64,
    seen_project: u64,
    seen_project_id: Option<String>,
    rebuilds: u64,
    catalog: Catalog,
    pending_events: Vec<WireEvent>,
}

impl<Port: ReaperPort> TimerLoop<Port> {
    /// Reads the project once and starts idle at the first song.
    pub fn new(mut port: Port) -> Self {
        project_identity::ensure(&mut port);
        let project_setlists = ProjectSetlists::read(&port);
        let songs = plan_songs(&port.regions(), &port.markers(), &project_setlists);
        let seen_change_count = port.change_count();
        let seen_project = port.project_token();
        let seen_project_id = project_identity::read(&port);
        let performance = Self::fresh(&port, songs.clone());
        let catalog = build_catalog(
            1,
            1,
            &project_setlists,
            &songs,
            &port.regions(),
            &port.markers(),
        );
        Self {
            port,
            performance,
            songs,
            project_setlists,
            seen_change_count,
            seen_project,
            seen_project_id,
            rebuilds: 0,
            catalog,
            pending_events: Vec::new(),
        }
    }

    /// Runs one timer tick.
    pub fn tick(&mut self) {
        self.refresh_if_changed();
        self.apply(Input::Observed {
            playing: self.port.transport() == reaper_port::Transport::Playing,
        });
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
        if let Command::SaveSetlist {
            id,
            name,
            entries,
            expected_revision,
        } = &command
        {
            let edit = SetlistEdit {
                id,
                name,
                entries,
                expected_revision: *expected_revision,
            };
            let saved = self.project_setlists.save(&mut self.port, edit);
            self.refresh_if_changed();
            return self.outcome(saved);
        }
        if let Command::DeleteSetlist {
            id,
            expected_revision,
        } = &command
        {
            let deleted = self
                .project_setlists
                .delete(&mut self.port, id, *expected_revision);
            self.refresh_if_changed();
            return self.outcome(deleted);
        }
        if let Command::SetActiveSetlist { id } = &command {
            let chosen = self
                .project_setlists
                .set_active(&mut self.port, id.as_deref());
            self.refresh_if_changed();
            return self.outcome(chosen);
        }
        let start = self.song_start();
        match to_input(command, start, self.performance.flags()) {
            Ok(input) => {
                self.command(input);
                Outcome::Done
            }
            Err(reason) => self.outcome(Err(reason)),
        }
    }

    /// `Done`, or a refusal that is also recorded as an event.
    fn outcome(&mut self, result: Result<(), &'static str>) -> Outcome {
        match result {
            Ok(()) => Outcome::Done,
            Err(reason) => {
                self.record(WireEvent::CommandRejected {
                    reason: reason.into(),
                });
                Outcome::Rejected {
                    reason: reason.into(),
                }
            }
        }
    }

    /// The state that changes all the time. The sender stamps `sequence` and `timestamp`.
    pub fn live(&self) -> Live {
        let flags = self.performance.flags();
        let index = self.performance.current_index();
        let song = |index: usize| u32::try_from(index).ok();
        Live {
            transport: match self.port.transport() {
                reaper_port::Transport::Stopped => Transport::Stopped,
                reaper_port::Transport::Playing => Transport::Playing,
                reaper_port::Transport::Paused => Transport::Paused,
            },
            position: self.port.position().get(),
            phase: map_phase(self.performance.phase()),
            setlist_id: self.catalog.active_setlist.clone(),
            current_song: index.and_then(song),
            next_song: index
                .map(|index| index + 1)
                .filter(|next| *next < self.catalog.songs.len())
                .and_then(song),
            autoplay: flags.autoplay,
            count_in: flags.count_in,
            record_armed: false,
            catalog_revision: self.catalog.revision,
            setlist_revision: self.catalog.setlist_revision,
            ..Live::default()
        }
    }

    /// What happened since the last call, oldest first.
    pub fn take_events(&mut self) -> Vec<WireEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// REAPER's clock in seconds.
    pub fn now(&self) -> f64 {
        self.port.now().get()
    }

    /// The songs and cues of the project; the revision rises whenever they change.
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// Where the performance is.
    pub fn phase(&self) -> Phase {
        self.performance.phase()
    }

    /// How many times an edit or a switch of project made the performance start over.
    pub fn rebuilds(&self) -> u64 {
        self.rebuilds
    }

    /// The port, for the test probe and the simulator.
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
        let project = self.port.project_token();
        let switched = project != self.seen_project;
        if !switched
            && change_count == self.seen_change_count
            && self.project_setlists.is_current(&self.port)
        {
            return;
        }
        self.seen_change_count = change_count;
        self.seen_project = project;
        project_identity::ensure(&mut self.port);
        let project_id = project_identity::read(&self.port);
        let switched = switched || project_id != self.seen_project_id;
        self.seen_project_id = project_id;
        let regions = self.port.regions();
        let markers = self.port.markers();
        let project_setlists = ProjectSetlists::read(&self.port);
        let songs = plan_songs(&regions, &markers, &project_setlists);
        self.refresh_catalog(&project_setlists, &songs, &regions, &markers);
        self.project_setlists = project_setlists;
        if switched {
            self.record(WireEvent::ProjectChanged);
        }
        if switched || songs != self.songs {
            self.performance = Self::fresh(&self.port, songs.clone());
            self.songs = songs;
            self.rebuilds += 1;
        }
    }

    /// Builds the catalog again; the revision rises when songs or cues changed, the setlist
    /// revision when the setlists or the played one changed.
    fn refresh_catalog(
        &mut self,
        project_setlists: &ProjectSetlists,
        songs: &[PlannedSong],
        regions: &[Region],
        markers: &[Marker],
    ) {
        let mut catalog = build_catalog(
            self.catalog.revision,
            self.catalog.setlist_revision,
            project_setlists,
            songs,
            regions,
            markers,
        );
        if catalog.songs != self.catalog.songs || catalog.cues != self.catalog.cues {
            catalog.revision += 1;
        }
        if catalog.setlists != self.catalog.setlists
            || catalog.active_setlist != self.catalog.active_setlist
        {
            catalog.setlist_revision += 1;
        }
        self.catalog = catalog;
    }

    fn record(&mut self, event: WireEvent) {
        if self.pending_events.len() >= PENDING_EVENTS {
            self.pending_events.remove(0);
        }
        self.pending_events.push(event);
    }

    fn apply(&mut self, input: Input) {
        let output = self.performance.step(input);
        for event in output.events {
            self.record(to_wire_event(event));
        }
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
mod project_switch_tests;
#[cfg(test)]
mod tests;
