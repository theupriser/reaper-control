use performance::{Flag, Input};
use shared_kernel::Seconds;

use crate::{ScenarioError, Step};

pub(super) fn secs(value: f64) -> Result<Seconds, ScenarioError> {
    Seconds::new(value).map_err(|_| ScenarioError::Invalid(format!("{value} is not a time")))
}

/// The input a command step stands for; `None` for time and checks.
pub(super) fn input_of(step: &Step) -> Result<Option<Input>, ScenarioError> {
    Ok(Some(match step {
        Step::Play => Input::Play,
        Step::Pause => Input::Pause,
        Step::Next => Input::Next,
        Step::Previous => Input::Previous,
        Step::Restart => Input::RestartSong,
        Step::Seek { position } => Input::Seek {
            position: secs(*position)?,
        },
        Step::SeekCue { position, lead_in } => Input::SeekCue {
            position: secs(*position)?,
            lead_in: secs(*lead_in)?,
        },
        Step::SetFlag { flag, enabled } => Input::SetFlag {
            flag: flag_named(flag)?,
            enabled: *enabled,
        },
        Step::Advance { .. } | Step::Expect(_) => return Ok(None),
    }))
}

fn flag_named(name: &str) -> Result<Flag, ScenarioError> {
    match name {
        "autoplay" => Ok(Flag::Autoplay),
        "count_in" => Ok(Flag::CountIn),
        other => Err(ScenarioError::Invalid(format!("unknown flag {other:?}"))),
    }
}
