mod apply_effect;
mod check;
mod event_name;
mod input_of;

use performance::{Flags, HandOverPolicy, Input, Output, Performance, PlannedSong, SongWindow};
use performance::{TempoMap, TimeSignature};
use shared_kernel::{Bpm, SongId};

use crate::{FakeReaper, ReaperPort, Region, Scenario, ScenarioError, ScenarioSong, Step, Trace};
use apply_effect::apply_effect;
use check::check;
use event_name::event_name;
use input_of::{input_of, secs};

/// How often the performance is ticked while time passes (REAPER's timer is faster).
const TICK: f64 = 0.05;
/// The longest stretch one `advance` step may cover.
const LONGEST_ADVANCE: f64 = 3600.0;

/// Plays a scenario against a `FakeReaper` and the real `Performance`.
pub struct ScenarioRunner {
    // Boxed: the two together are over 200 bytes.
    reaper: Box<FakeReaper>,
    performance: Box<Performance>,
    trace: Trace,
    events: Vec<String>,
}

impl ScenarioRunner {
    /// Runs every step; the first failed expectation stops the run.
    pub fn run(scenario: &Scenario) -> Result<Trace, ScenarioError> {
        let mut runner = Self::new(scenario)?;
        for (index, step) in scenario.steps.iter().enumerate() {
            runner.step(index + 1, step)?;
        }
        Ok(runner.trace)
    }

    fn new(scenario: &Scenario) -> Result<Self, ScenarioError> {
        let planned = scenario
            .songs
            .iter()
            .map(planned_song)
            .collect::<Result<Vec<_>, _>>()?;
        let regions = scenario
            .songs
            .iter()
            .map(region)
            .collect::<Result<Vec<_>, _>>()?;
        let tempo = Bpm::new(120.0).map_err(|e| ScenarioError::Invalid(e.to_string()))?;
        let reaper = FakeReaper::new(
            regions,
            Vec::new(),
            TempoMap::constant(tempo, TimeSignature::common()),
        );
        let flags = Flags {
            autoplay: scenario.autoplay,
            count_in: scenario.count_in,
        };
        Ok(Self {
            reaper: Box::new(reaper),
            performance: Box::new(Performance::new(planned, flags, HandOverPolicy::default())),
            trace: Trace::default(),
            events: Vec::new(),
        })
    }

    fn step(&mut self, number: usize, step: &Step) -> Result<(), ScenarioError> {
        match step {
            Step::Advance { seconds } => self.advance(*seconds),
            Step::Expect(expect) => {
                let result = check(
                    expect,
                    &self.performance,
                    self.reaper.as_ref(),
                    &self.events,
                );
                self.events.clear();
                result.map_err(|message| ScenarioError::Failed {
                    step: number,
                    message,
                })
            }
            command => {
                if let Some(input) = input_of(command)? {
                    self.feed(input);
                }
                Ok(())
            }
        }
    }

    fn advance(&mut self, seconds: f64) -> Result<(), ScenarioError> {
        if !(0.0..=LONGEST_ADVANCE).contains(&seconds) {
            return Err(ScenarioError::Invalid(format!(
                "cannot advance {seconds} s"
            )));
        }
        let mut left = seconds;
        while left > 1e-9 {
            let span = left.min(TICK);
            self.reaper.advance(secs(span)?);
            self.feed(Input::Tick {
                now: self.reaper.now(),
                position: self.reaper.position(),
            });
            left -= span;
        }
        Ok(())
    }

    fn feed(&mut self, input: Input) {
        let quiet = matches!(input, Input::Tick { .. });
        let output = self.performance.step(input);
        for effect in &output.effects {
            apply_effect(self.reaper.as_mut(), *effect);
        }
        self.events.extend(output.events.iter().map(event_name));
        if !(quiet && output == Output::default()) {
            self.record(&input, &output);
        }
    }

    fn record(&mut self, input: &Input, output: &Output) {
        let label = match input {
            Input::Tick { .. } => "tick".to_string(),
            other => format!("{other:?}"),
        };
        self.trace.push(format!(
            "t={:.3} pos={:.3} {label} -> effects {:?} events {:?}",
            self.reaper.now().get(),
            self.reaper.position().get(),
            output.effects,
            output.events,
        ));
    }
}

fn planned_song(song: &ScenarioSong) -> Result<PlannedSong, ScenarioError> {
    let window = SongWindow::new(secs(song.start)?, secs(song.end)?)
        .map_err(|_| ScenarioError::Invalid(format!("song {} ends before it starts", song.id)))?;
    Ok(PlannedSong {
        song_id: song.id.clone(),
        window,
        hard_stop: song.hard_stop,
        hard_stop_marker: None,
    })
}

fn region(song: &ScenarioSong) -> Result<Region, ScenarioError> {
    Ok(Region {
        id: SongId::new(song.id.clone()),
        number: 0,
        name: song.id.clone(),
        start: secs(song.start)?,
        end: secs(song.end)?,
    })
}
