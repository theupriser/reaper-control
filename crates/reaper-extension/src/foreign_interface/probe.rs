use std::path::PathBuf;

use performance::Input;
use reaper_medium::ProjectContext;
use reaper_port::ReaperPort;
use shared_kernel::Seconds;

use super::reaper_rs_adapter::ReaperRsAdapter;
use crate::log::Log;
use timer_loop::TimerLoop;

/// Test builds only: runs the commands in `probe-command` (one per line) against the adapter and
/// logs what it reads, so the adapter can be checked in a real REAPER without a link.
#[derive(Debug)]
pub(super) struct Probe {
    file: PathBuf,
}

impl Probe {
    pub(super) fn new(directory: PathBuf) -> Self {
        Self {
            file: directory.join("probe-command"),
        }
    }

    pub(super) fn run(&self, timer_loop: &mut TimerLoop<ReaperRsAdapter>, log: &Log) {
        let Ok(commands) = std::fs::read_to_string(&self.file) else {
            return;
        };
        let _ = std::fs::remove_file(&self.file);
        for line in commands.lines() {
            let words: Vec<&str> = line.split_whitespace().collect();
            log.line(&format!("probe> {line}"));
            match words.as_slice() {
                ["perf", rest @ ..] => Self::perf(rest, timer_loop, log),
                _ => Self::command(&words, timer_loop.port_mut(), log),
            }
        }
    }

    /// Drives the performance the way the link will: a command through the timer loop.
    fn perf(words: &[&str], timer_loop: &mut TimerLoop<ReaperRsAdapter>, log: &Log) {
        match words {
            ["play"] => timer_loop.command(Input::Play),
            ["pause"] => timer_loop.command(Input::Pause),
            ["next"] => timer_loop.command(Input::Next),
            ["previous"] => timer_loop.command(Input::Previous),
            ["restart"] => timer_loop.command(Input::RestartSong),
            ["phase"] => log.line(&format!(
                "probe: phase {:?}, rebuilds {}",
                timer_loop.phase(),
                timer_loop.rebuilds()
            )),
            ["catalog"] => {
                let catalog = timer_loop.catalog();
                let songs: Vec<&str> = catalog
                    .songs
                    .iter()
                    .map(|song| song.name.as_str())
                    .collect();
                log.line(&format!(
                    "probe: catalog revision {}, setlist revision {}, songs {songs:?}, played setlist {:?}, current song {:?}",
                    catalog.revision,
                    catalog.setlist_revision,
                    catalog.active_setlist,
                    timer_loop.live().current_song
                ));
            }
            _ => log.line("probe: unknown perf command"),
        }
    }

    fn command(words: &[&str], adapter: &mut ReaperRsAdapter, log: &Log) {
        match words {
            ["dump"] => Self::dump(adapter, log),
            ["play"] => adapter.play(),
            ["pause"] => adapter.pause(),
            ["seek", time] => match time.parse().ok().and_then(|t| Seconds::new(t).ok()) {
                Some(time) => adapter.seek(time),
                None => log.line("probe: not a time"),
            },
            ["count-in", state] => adapter.set_count_in(*state == "on"),
            ["ext-get", section, key] => {
                log.line(&format!("probe: {:?}", adapter.ext_state(section, key)));
            }
            ["ext-set", section, key, value @ ..] => {
                adapter.set_ext_state(section, key, &value.join(" "));
            }
            ["bar", time] => match time.parse().ok().and_then(|t| Seconds::new(t).ok()) {
                Some(time) => log.line(&format!(
                    "probe: the bar before {} s is {} s",
                    time.get(),
                    adapter.tempo_map().bars_before(time, 1).get()
                )),
                None => log.line("probe: not a time"),
            },
            ["tempo-marker", time, bpm, beats, unit] => {
                let numbers = (time.parse(), bpm.parse(), beats.parse(), unit.parse());
                if let (Ok(time), Ok(bpm), Ok(beats), Ok(unit)) = numbers {
                    let done = Self::add_tempo_marker(adapter, time, bpm, beats, unit);
                    log.line(&format!("probe: tempo marker added: {done}"));
                }
            }
            ["action", id] => {
                if let Ok(id) = id.parse() {
                    adapter.reaper().main_on_command_ex(
                        reaper_medium::CommandId::new(id),
                        0,
                        ProjectContext::CurrentProject,
                    );
                }
            }
            ["toggle-state", id] => {
                if let Ok(id) = id.parse() {
                    let state = adapter.reaper().get_toggle_command_state_ex(
                        reaper_medium::SectionId::new(0),
                        reaper_medium::CommandId::new(id),
                    );
                    log.line(&format!("probe: action {id} toggle state {state:?}"));
                }
            }
            ["find-action", text] => Self::find_action(adapter, text, log),
            ["config", name] => {
                let value = adapter.reaper().get_config_var(*name).map(|found| {
                    // SAFETY: REAPER keeps the value alive; a preference of 4 bytes is an i32.
                    (found.size, unsafe { *found.value.as_ptr().cast::<i32>() })
                });
                log.line(&format!("probe: config {name} = {value:?}"));
            }
            ["config-set", name, value] => {
                if let (Some(found), Ok(value)) =
                    (adapter.reaper().get_config_var(*name), value.parse::<i32>())
                {
                    // SAFETY: as for `config`; this changes the isolated test setup only.
                    unsafe { *found.value.as_ptr().cast::<i32>() = value };
                }
            }
            ["project-config", name, rest @ ..] => {
                let value = rest.first().and_then(|text| text.parse::<i32>().ok());
                Self::project_config(adapter, name, value, log);
            }
            ["audio"] => log.line(&format!(
                "probe: audio running {}",
                adapter.reaper().audio_is_running()
            )),
            ["measure", index] => Self::measure(adapter, index, log),
            _ => log.line("probe: unknown command"),
        }
    }

    fn add_tempo_marker(
        adapter: &ReaperRsAdapter,
        time: f64,
        bpm: f64,
        beats: i32,
        unit: i32,
    ) -> bool {
        // SAFETY: plain numbers; a null project is the current project.
        unsafe {
            adapter.reaper().low().SetTempoTimeSigMarker(
                std::ptr::null_mut(),
                -1,
                time,
                -1,
                -1.0,
                bpm,
                beats,
                unit,
                false,
            )
        }
    }

    /// REAPER's own answer for a measure, to compare the adapter's tempo map with.
    fn measure(adapter: &ReaperRsAdapter, index: &str, log: &Log) {
        let Ok(index) = index.parse() else {
            return;
        };
        let measure = adapter
            .reaper()
            .time_map_get_measure_info(ProjectContext::CurrentProject, index);
        log.line(&format!(
            "probe: reaper measure {index} starts at {:.6} s, {}/{} at {} bpm",
            measure.start_time.get(),
            measure.time_signature.numerator,
            measure.time_signature.denominator,
            measure.tempo.get()
        ));
    }

    /// Lists the main-section actions whose name contains `text`, with their toggle state.
    fn find_action(adapter: &ReaperRsAdapter, text: &str, log: &Log) {
        let low = adapter.reaper().low();
        let section = low.SectionFromUniqueID(0);
        for id in 40000..=60000 {
            // SAFETY: REAPER returns null or a NUL-terminated name it owns.
            let name = unsafe {
                let name = low.kbd_getTextFromCmd(id, section);
                if name.is_null() {
                    continue;
                }
                std::ffi::CStr::from_ptr(name)
                    .to_string_lossy()
                    .into_owned()
            };
            if name.to_lowercase().contains(&text.to_lowercase()) {
                let state = adapter.reaper().get_toggle_command_state_ex(
                    reaper_medium::SectionId::new(0),
                    reaper_medium::CommandId::new(id as u32),
                );
                log.line(&format!("probe: action {id} {name:?} {state:?}"));
            }
        }
    }

    /// Reads (and with a value, writes) a setting that belongs to the current project.
    fn project_config(adapter: &ReaperRsAdapter, name: &str, value: Option<i32>, log: &Log) {
        let reaper = adapter.reaper();
        let Some(offset) = reaper.project_config_var_get_offs(name) else {
            return log.line(&format!("probe: no project setting {name}"));
        };
        let Some(address) =
            reaper.project_config_var_addr(ProjectContext::CurrentProject, offset.offset)
        else {
            return log.line("probe: no address");
        };
        let pointer = address.as_ptr().cast::<i32>();
        // SAFETY: REAPER owns the setting for the session; settings read here are 4 bytes.
        unsafe {
            if let Some(value) = value {
                *pointer = value;
            }
            log.line(&format!(
                "probe: project {name} = {} (size {})",
                *pointer, offset.size
            ));
        }
    }

    fn dump(adapter: &ReaperRsAdapter, log: &Log) {
        log.line(&format!(
            "probe: now {:.3} position {:.3} transport {:?} count_in {}",
            adapter.now().get(),
            adapter.position().get(),
            adapter.transport(),
            adapter.count_in()
        ));
        for region in adapter.regions() {
            log.line(&format!("probe: {region:?}"));
        }
        for marker in adapter.markers() {
            log.line(&format!("probe: {marker:?}"));
        }
        log.line(&format!("probe: {:?}", adapter.tempo_map()));
    }
}
