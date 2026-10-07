//! Spike S2: timing of hand-overs. Reads a one-line command from `<resource>/RC2/s2-run.txt`
//! (`seek <lead_ms> [start_s]` or `stopseek <lead_ms> [start_s]`), then plays the click project from 6 s and, when the
//! play position comes within `lead_ms` of the end of Song A (10 s), jumps to the start of Song B
//! (12 s). Every main-thread tick is logged (`T`), plus the trigger (`E`), to `spike-s2.log`.
#![allow(unsafe_code)] // spike only

use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{Duration, Instant};

use reaper_low::PluginContext;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::{
    ControlSurface, MainThreadScope, PositionInSeconds, ProjectContext, Reaper, ReaperSession,
    SetEditCurPosOptions,
};

const SONG_A_END: f64 = 10.0;
const SONG_B_START: f64 = 12.0;
const RUN_AFTER_TRIGGER: Duration = Duration::from_secs(3);

#[derive(Clone, Copy, Debug, PartialEq)]
enum Method {
    Seek,
    StopSeek,
}

#[derive(Debug)]
struct Run {
    id: u32,
    method: Method,
    lead: f64,
    began: Instant,
    started_play: bool,
    triggered_at: Option<Instant>,
}

#[derive(Debug)]
struct Spike {
    reaper: Reaper<MainThreadScope>,
    cmd_path: String,
    log_path: String,
    last_poll: Instant,
    last_tick: Instant,
    runs: u32,
    run: Option<Run>,
}

impl Spike {
    fn log(&self, line: String) {
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&self.log_path) {
            let _ = writeln!(f, "{line}");
        }
    }

    fn pos(&self) -> f64 {
        self.reaper.get_play_position_2_ex(ProjectContext::CurrentProject).get()
    }

    fn seek_to(&self, t: f64, seek_play: bool) {
        self.reaper.set_edit_curs_pos_2(
            ProjectContext::CurrentProject,
            PositionInSeconds::new(t).unwrap_or_default(),
            SetEditCurPosOptions { move_view: false, seek_play },
        );
    }

    fn poll_command(&mut self) {
        let Ok(text) = std::fs::read_to_string(&self.cmd_path) else { return };
        let _ = std::fs::remove_file(&self.cmd_path);
        let mut parts = text.split_whitespace();
        let method = match parts.next() {
            Some("seek") => Method::Seek,
            Some("stopseek") => Method::StopSeek,
            _ => return,
        };
        let lead_ms: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let start_at: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(6.0);
        self.runs += 1;
        let project = ProjectContext::CurrentProject;
        self.reaper.on_stop_button_ex(project);
        self.seek_to(start_at, false);
        self.log(format!("BEGIN run={} method={method:?} lead_ms={lead_ms} start_at={start_at}", self.runs));
        self.run = Some(Run {
            id: self.runs,
            method,
            lead: lead_ms / 1000.0,
            began: Instant::now(),
            started_play: false,
            triggered_at: None,
        });
    }

    fn step(&mut self, dt_ms: f64) {
        let project = ProjectContext::CurrentProject;
        let pos = self.pos();
        let state = self.reaper.get_play_state_ex(project);
        let Some(run) = self.run.as_mut() else { return };
        let ms = run.began.elapsed().as_secs_f64() * 1000.0;
        let (id, method, lead) = (run.id, run.method, run.lead);
        if !run.started_play {
            // give the stop/seek one tick to settle, then press play
            if ms > 100.0 {
                run.started_play = true;
                self.reaper.on_play_button_ex(project);
            }
            return;
        }
        let trigger_wall = run.triggered_at;
        let fire = trigger_wall.is_none() && state.is_playing && pos + lead >= SONG_A_END;
        if fire {
            run.triggered_at = Some(Instant::now());
        }
        let done = trigger_wall.is_some_and(|t| t.elapsed() > RUN_AFTER_TRIGGER);
        self.log(format!("T run={id} ms={ms:.1} dt={dt_ms:.1} pos={pos:.4} playing={}", state.is_playing));
        if fire {
            let before = Instant::now();
            match method {
                Method::Seek => self.seek_to(SONG_B_START, true),
                Method::StopSeek => {
                    self.reaper.on_stop_button_ex(project);
                    self.seek_to(SONG_B_START, false);
                    self.reaper.on_play_button_ex(project);
                }
            }
            let call_ms = before.elapsed().as_secs_f64() * 1000.0;
            self.log(format!("E run={id} ms={ms:.1} trigger pos_before={pos:.4} call_ms={call_ms:.3}"));
        }
        if done {
            self.reaper.on_stop_button_ex(project);
            self.log(format!("END run={id}"));
            self.run = None;
        }
    }
}

impl ControlSurface for Spike {
    fn run(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f64() * 1000.0;
        self.last_tick = now;
        if now.duration_since(self.last_poll) >= Duration::from_millis(500) {
            self.last_poll = now;
            if self.run.is_none() {
                self.poll_command();
            }
        }
        self.step(dt);
    }
}

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    let mut session = ReaperSession::load(context);
    let reaper = session.reaper().clone();
    let resource = reaper.get_resource_path(|p| p.to_string());
    let dir = format!("{resource}/RC2");
    let _ = std::fs::create_dir_all(&dir);
    reaper.show_console_msg(format!("RC2 S2: loaded, command file {dir}/s2-run.txt\n"));
    let spike = Spike {
        reaper,
        cmd_path: format!("{dir}/s2-run.txt"),
        log_path: format!("{dir}/spike-s2.log"),
        last_poll: Instant::now(),
        last_tick: Instant::now(),
        runs: 0,
        run: None,
    };
    session.plugin_register_add_csurf_inst(Box::new(spike))?;
    Box::leak(Box::new(session));
    Ok(())
}
