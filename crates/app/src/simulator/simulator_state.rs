//! The simulated extension: the real timer loop over `FakeReaper`, publishing like the link does.

use link::LinkEvent;
use protocol::message::Outcome;
use protocol::{Command, EventRecord, Live};
use reaper_port::FakeReaper;
use shared_kernel::Seconds;
use timer_loop::TimerLoop;

use crate::link_pipeline::LinkPipeline;

/// The simulated REAPER moves in steps of this many seconds, like the extension's timer.
pub(super) const STEP_SECONDS: f64 = 0.05;

pub(super) struct SimulatorState {
    timer_loop: TimerLoop<FakeReaper>,
    pipeline: LinkPipeline,
    next_command: u64,
    last_event: u64,
    sequence: u64,
    published: Option<Live>,
    published_revisions: Option<(u64, u64)>,
}

impl SimulatorState {
    /// Connects the pipeline and publishes the first catalog and state.
    pub(super) fn start(reaper: FakeReaper, pipeline: LinkPipeline) -> Self {
        let mut state = Self {
            timer_loop: TimerLoop::new(reaper),
            pipeline,
            next_command: 1,
            last_event: 0,
            sequence: 0,
            published: None,
            published_revisions: None,
        };
        state.pipeline.deliver(LinkEvent::Connected {
            extension_version: "simulator".into(),
        });
        state.publish();
        state
    }

    /// Carries out a command and answers it at once, as the extension does; a refusal shows up
    /// as an event.
    pub(super) fn command(&mut self, command: Command) -> u64 {
        let id = self.next_command;
        self.next_command += 1;
        self.timer_loop.link_command(command);
        self.pipeline.deliver(LinkEvent::Ack {
            id,
            outcome: Outcome::Done,
        });
        self.publish();
        id
    }

    /// One timer tick after `span` seconds of simulated time.
    pub(super) fn step(&mut self, span: f64) {
        self.timer_loop
            .port_mut()
            .advance(Seconds::new(span).unwrap_or(Seconds::ZERO));
        self.timer_loop.tick();
        self.publish();
    }

    fn publish(&mut self) {
        for event in self.timer_loop.take_events() {
            self.last_event += 1;
            self.pipeline.deliver(LinkEvent::Event(EventRecord {
                id: self.last_event,
                event,
            }));
        }
        let catalog = self.timer_loop.catalog();
        let revisions = (catalog.revision, catalog.setlist_revision);
        if self.published_revisions != Some(revisions) {
            self.published_revisions = Some(revisions);
            self.pipeline.deliver(LinkEvent::Catalog(catalog.clone()));
        }
        let live = self.timer_loop.live();
        if self.published.as_ref() != Some(&live) {
            self.sequence += 1;
            self.published = Some(live.clone());
            self.pipeline.deliver(LinkEvent::Live(Live {
                sequence: self.sequence,
                timestamp: self.timer_loop.now(),
                ..live
            }));
        }
    }
}
