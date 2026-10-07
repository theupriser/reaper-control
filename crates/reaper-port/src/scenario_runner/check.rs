use performance::Performance;

use crate::{Expectation, ReaperPort};

/// How close a position must be to count as equal.
const POSITION_TOLERANCE: f64 = 0.001;

/// Compares the state with an expectation; the error says what differs.
pub(super) fn check(
    expect: &Expectation,
    performance: &Performance,
    reaper: &impl ReaperPort,
    events: &[String],
) -> Result<(), String> {
    if let Some(phase) = &expect.phase {
        same("phase", phase, &format!("{:?}", performance.phase()))?;
    }
    if let Some(song) = &expect.song {
        let current = performance.current().map(|s| s.song_id.clone());
        same("song", song, &current.unwrap_or_else(|| "none".into()))?;
    }
    if let Some(transport) = &expect.transport {
        same("transport", transport, &format!("{:?}", reaper.transport()))?;
    }
    if let Some(count_in) = expect.count_in {
        same("count-in", &count_in, &reaper.count_in())?;
    }
    if let Some(position) = expect.position {
        let got = reaper.position().get();
        if (got - position).abs() > POSITION_TOLERANCE {
            return Err(format!("expected position {position}, got {got:.3}"));
        }
    }
    if let Some(wanted) = &expect.events {
        same("events", &wanted.join(", "), &events.join(", "))?;
    }
    Ok(())
}

fn same<T: PartialEq + std::fmt::Display + ?Sized>(
    what: &str,
    expected: &T,
    got: &T,
) -> Result<(), String> {
    if expected == got {
        Ok(())
    } else {
        Err(format!("expected {what} [{expected}], got [{got}]"))
    }
}
