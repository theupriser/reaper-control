use performance::{TempoMap, TempoSegment, TimeSignature};
use reaper_medium::{MainThreadScope, ProjectContext, Reaper};
use shared_kernel::{Bpm, Seconds};

/// A tempo or signature change as REAPER stores it; a zero signature means "unchanged".
struct Change {
    time: f64,
    bpm: f64,
    beats: i32,
    unit: i32,
}

/// The project's tempo and signature changes. The segment at time zero comes from the measure
/// REAPER reports there, so a project without any change still has a map. A tempo that glides
/// (linear change) counts as the tempo it starts with.
pub(super) fn read(reaper: &Reaper<MainThreadScope>) -> TempoMap {
    let start = start_segment(reaper);
    let mut segments = vec![start];
    for change in changes(reaper) {
        let Some(previous) = segments.last().copied() else {
            break;
        };
        let Ok(time) = Seconds::new(change.time) else {
            continue;
        };
        let signature = u32::try_from(change.beats)
            .ok()
            .zip(u32::try_from(change.unit).ok())
            .and_then(|(beats, unit)| TimeSignature::new(beats, unit).ok())
            .unwrap_or_else(|| previous.signature());
        let Ok(bpm) = Bpm::new(change.bpm) else {
            continue;
        };
        let segment = TempoSegment::new(time, bpm, signature);
        if time <= previous.start() {
            segments.pop();
        }
        segments.push(segment);
    }
    TempoMap::new(segments).unwrap_or_else(|_| TempoMap::constant(start.bpm(), start.signature()))
}

fn start_segment(reaper: &Reaper<MainThreadScope>) -> TempoSegment {
    let measure = reaper.time_map_get_measure_info(ProjectContext::CurrentProject, 0);
    let signature = TimeSignature::new(
        measure.time_signature.numerator.get(),
        measure.time_signature.denominator.get(),
    )
    .unwrap_or_else(|_| TimeSignature::common());
    let bpm = Bpm::new(measure.tempo.get()).unwrap_or(Bpm::FALLBACK);
    TempoSegment::new(Seconds::ZERO, bpm, signature)
}

fn changes(reaper: &Reaper<MainThreadScope>) -> Vec<Change> {
    let project = ProjectContext::CurrentProject;
    let count = reaper.count_tempo_time_sig_markers(project);
    (0..count)
        .filter_map(|index| {
            let (mut time, mut bpm) = (0.0_f64, 0.0_f64);
            let (mut measure, mut beats, mut unit) = (0_i32, 0_i32, 0_i32);
            let (mut beat, mut linear) = (0.0_f64, false);
            // SAFETY: every pointer is to a local that lives through the call.
            let found = unsafe {
                reaper.low().GetTempoTimeSigMarker(
                    project.to_raw(),
                    i32::try_from(index).ok()?,
                    &mut time,
                    &mut measure,
                    &mut beat,
                    &mut bpm,
                    &mut beats,
                    &mut unit,
                    &mut linear,
                )
            };
            found.then_some(Change {
                time,
                bpm,
                beats,
                unit,
            })
        })
        .collect()
}
