//! Spike S1: minimal reaper-rs extension. On load it prints to REAPER's console. A control
//! surface `run()` (REAPER calls it ~30 Hz on the main thread) measures the tick interval and
//! reads play state, position, markers/regions and tempo, reporting every few seconds to the
//! console and to `<resource>/RC2/spike-s1.log`.
#![allow(unsafe_code)] // spike only; the real extension confines unsafe to one module (S-9)

use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{Duration, Instant};

use reaper_low::PluginContext;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::{ControlSurface, MainThreadScope, ProjectContext, Reaper, ReaperSession};

const REPORT_EVERY: Duration = Duration::from_secs(3);

#[derive(Debug)]
struct Spike {
    reaper: Reaper<MainThreadScope>,
    log_path: String,
    last_tick: Instant,
    last_report: Instant,
    ticks: u64,
    window_ticks: u64,
    min_ms: f64,
    max_ms: f64,
    sum_ms: f64,
    dumped_count: u32,
}

impl Spike {
    /// Writes every region and marker (index, id, position, end, name) to the log, once.
    fn dump_project(&mut self) {
        let project = ProjectContext::CurrentProject;
        let counts = self.reaper.count_project_markers(project);
        let mut lines = vec![format!(
            "--- project dump: {} regions, {} markers ---",
            counts.region_count, counts.marker_count
        )];
        for index in 0..counts.total_count {
            let line = self
                .reaper
                .enum_project_markers_3(project, index, |item| match item {
                    Some(m) => {
                        let kind = if m.region_end_position.is_some() {
                            "region"
                        } else {
                            "marker"
                        };
                        let end = m.region_end_position.map(|e| e.get()).unwrap_or(0.0);
                        format!(
                            "{index:>3} {kind} id={} start={:.3} end={:.3} name={:?}",
                            m.id.get(),
                            m.position.get(),
                            end,
                            m.name.to_str()
                        )
                    }
                    None => format!("{index:>3} (none)"),
                });
            lines.push(line);
        }
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            for l in &lines {
                let _ = writeln!(f, "{l}");
            }
        }
        self.reaper.show_console_msg(format!(
            "RC2 S1: dumped {} markers/regions to log\n",
            counts.total_count
        ));
    }

    fn report(&mut self) {
        let project = ProjectContext::CurrentProject;
        let state = self.reaper.get_play_state_ex(project);
        let position = self.reaper.get_play_position_2_ex(project).get();
        let counts = self.reaper.count_project_markers(project);
        let bpm = self
            .reaper
            .time_map_2_get_divided_bpm_at_time(
                project,
                self.reaper.get_play_position_2_ex(project),
            )
            .get();
        let avg = if self.window_ticks > 0 {
            self.sum_ms / self.window_ticks as f64
        } else {
            0.0
        };
        let line = format!(
            "ticks={} window_ticks={} dt_ms[min/avg/max]={:.1}/{:.1}/{:.1} playing={} paused={} rec={} pos={:.3}s regions={} markers={} bpm={:.2}",
            self.ticks,
            self.window_ticks,
            self.min_ms,
            avg,
            self.max_ms,
            state.is_playing,
            state.is_paused,
            state.is_recording,
            position,
            counts.region_count,
            counts.marker_count,
            bpm
        );
        self.reaper.show_console_msg(format!("RC2 S1: {line}\n"));
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = writeln!(f, "{line}");
        }
        self.window_ticks = 0;
        self.min_ms = f64::MAX;
        self.max_ms = 0.0;
        self.sum_ms = 0.0;
    }
}

impl ControlSurface for Spike {
    fn run(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f64() * 1000.0;
        self.last_tick = now;
        self.ticks += 1;
        if self.ticks > 1 {
            self.window_ticks += 1;
            self.sum_ms += dt;
            self.min_ms = self.min_ms.min(dt);
            self.max_ms = self.max_ms.max(dt);
        }
        if now.duration_since(self.last_report) >= REPORT_EVERY {
            self.last_report = now;
            // Dump again whenever the number of markers/regions changed (project finished loading).
            let total = self
                .reaper
                .count_project_markers(ProjectContext::CurrentProject)
                .total_count;
            if total != self.dumped_count {
                self.dumped_count = total;
                self.dump_project();
            }
            self.report();
        }
    }
}

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    let mut session = ReaperSession::load(context);
    let reaper = session.reaper().clone();
    let version = reaper.get_app_version();
    let resource = reaper.get_resource_path(|p| p.to_string());
    let arch = std::env::consts::ARCH;
    reaper.show_console_msg(format!(
        "RC2 S1: extension loaded. REAPER {version}, extension arch {arch}, resource path {resource}\n"
    ));
    let dir = format!("{resource}/RC2");
    let _ = std::fs::create_dir_all(&dir);
    let spike = Spike {
        reaper,
        log_path: format!("{dir}/spike-s1.log"),
        last_tick: Instant::now(),
        last_report: Instant::now(),
        ticks: 0,
        window_ticks: 0,
        min_ms: f64::MAX,
        max_ms: 0.0,
        sum_ms: 0.0,
        dumped_count: 0,
    };
    session.plugin_register_add_csurf_inst(Box::new(spike))?;
    // Keep the session (and with it the registration) alive for the lifetime of the process.
    Box::leak(Box::new(session));
    Ok(())
}
