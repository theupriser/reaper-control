//! Spike S7: crash containment. Fault injection by trigger files in `<resource>/RC2/`:
//! `s7-panic-main` panics inside the main-thread tick, `s7-panic-thread` panics a worker
//! thread, `s7-segv-main` raises a real signal (the case no Rust code can contain).
//! A panic hook flips `FAULTED`; the tick then does nothing but keep a heartbeat.
//! A `s7-running` marker is written at start and removed on clean shutdown; if it exists at
//! start, the extension starts disabled (safe mode).
#![allow(unsafe_code)] // spike only
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use reaper_low::PluginContext;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::{
    ControlSurface, ExtSetProjectMarkerChangeArgs, MainThreadScope, OnAudioBuffer, OnAudioBufferArgs,
    ProjectContext, Reaper, ReaperSession,
};

const REPORT_EVERY: Duration = Duration::from_secs(3);

static FAULTED: AtomicBool = AtomicBool::new(false);
static ARM_LOAD: AtomicBool = AtomicBool::new(false);
static ARM_AUDIO: AtomicBool = AtomicBool::new(false);
static AUDIO_CALLS: AtomicU64 = AtomicU64::new(0);

struct AudioHook;

impl OnAudioBuffer for AudioHook {
    fn call(&mut self, args: OnAudioBufferArgs) {
        if args.is_post {
            return;
        }
        AUDIO_CALLS.fetch_add(1, Relaxed);
        if ARM_AUDIO.swap(false, Relaxed) {
            panic!("injected panic in audio hook");
        }
    }
}
static DIR: OnceLock<PathBuf> = OnceLock::new();

fn dir() -> PathBuf {
    DIR.get().cloned().unwrap_or_default()
}

fn log(line: &str) {
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir().join("spike-s7.log"))
    {
        let _ = writeln!(f, "{line}");
    }
}

/// True once if the trigger file exists (and consumes it).
fn triggered(name: &str) -> bool {
    std::fs::remove_file(dir().join(name)).is_ok()
}

unsafe extern "C" {
    fn atexit(cb: extern "C" fn()) -> i32;
}

extern "C" fn on_exit() {
    let _ = std::fs::remove_file(dir().join("s7-running"));
    log("atexit: marker removed");
}

impl Drop for Spike {
    fn drop(&mut self) {
        log("drop: Spike dropped");
    }
}

#[derive(Debug)]
struct Spike {
    reaper: Reaper<MainThreadScope>,
    calls: u64,
    ticks_alive: Arc<AtomicU64>,
    last_report: Instant,
}

impl ControlSurface for Spike {
    fn run(&mut self) {
        self.calls += 1;
        if !FAULTED.load(Relaxed) {
            self.ticks_alive.fetch_add(1, Relaxed);
            if triggered("s7-panic-main") {
                log("injecting panic in main-thread tick");
                panic!("injected panic in tick");
            }
            if triggered("s7-segv-main") {
                log("injecting SIGSEGV in main-thread tick");
                // SAFETY: deliberate null write to raise a real SIGSEGV.
                unsafe { std::ptr::write_volatile(std::ptr::null_mut::<u8>(), 1) };
            }
            if let Ok(path) = std::fs::read_to_string(dir().join("s7-open")) {
                let _ = std::fs::remove_file(dir().join("s7-open"));
                log(&format!("opening project {}", path.trim()));
                let c = std::ffi::CString::new(path.trim()).unwrap_or_default();
                // SAFETY: main-thread call with a valid NUL-terminated path.
                unsafe { self.reaper.low().Main_openProject(c.as_ptr()) };
                log("open returned");
            }
            if triggered("s7-play") {
                // SAFETY: main-thread action call, action 1007 = Transport: Play.
                unsafe { self.reaper.low().Main_OnCommand(1007, 0) };
                log("started playback (action 1007)");
            }
            if triggered("s7-panic-load") {
                log("armed: panic in next project-marker change (fires on project load)");
                ARM_LOAD.store(true, Relaxed);
            }
            if triggered("s7-panic-audio") {
                log("armed: panic in audio hook");
                ARM_AUDIO.store(true, Relaxed);
            }
        }
        if self.last_report.elapsed() >= REPORT_EVERY {
            self.last_report = Instant::now();
            let state = self
                .reaper
                .get_play_state_ex(ProjectContext::CurrentProject);
            let pos = self
                .reaper
                .get_play_position_2_ex(ProjectContext::CurrentProject)
                .get();
            // SAFETY: plain read of REAPER's transport state on the main thread.
            let raw = unsafe { self.reaper.low().GetPlayStateEx(std::ptr::null_mut()) };
            log(&format!(
                "calls={} alive_ticks={} faulted={} playing={} raw_state={raw} audio_calls={} pos={pos:.2}",
                self.calls,
                self.ticks_alive.load(Relaxed),
                FAULTED.load(Relaxed),
                state.is_playing,
                AUDIO_CALLS.load(Relaxed)
            ));
        }
    }

    fn ext_set_project_marker_change(&self, _: ExtSetProjectMarkerChangeArgs) -> i32 {
        if ARM_LOAD.swap(false, Relaxed) {
            panic!("injected panic in project-marker change");
        }
        0
    }

    fn close_no_reset(&self) {
        let _ = std::fs::remove_file(dir().join("s7-running"));
        log("clean shutdown: marker removed");
    }
}

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    let mut session = ReaperSession::load(context);
    let reaper = session.reaper().clone();
    let resource = reaper.get_resource_path(|p| p.to_string());
    let rc2 = PathBuf::from(resource).join("RC2");
    let _ = std::fs::create_dir_all(&rc2);
    let _ = DIR.set(rc2.clone());

    std::panic::set_hook(Box::new(|info| {
        FAULTED.store(true, Relaxed);
        log(&format!("PANIC -> Faulted: {info}"));
    }));

    let marker = rc2.join("s7-running");
    if marker.exists() {
        log(
            "SAFE MODE: running marker found (REAPER did not shut down cleanly); extension stays disabled",
        );
        reaper.show_console_msg("RC2 S7: safe mode, extension disabled\n");
        return Ok(());
    }
    std::fs::write(&marker, b"1")?;
    log("started, marker written");
    // SAFETY: registers a plain extern "C" fn that only touches the filesystem.
    let rc = unsafe { atexit(on_exit) };
    log(&format!("atexit registered rc={rc}"));

    let ticks_alive = Arc::new(AtomicU64::new(0));
    thread::spawn(|| {
        loop {
            thread::sleep(Duration::from_millis(50));
            if triggered("s7-panic-thread") {
                log("injecting panic in worker thread");
                panic!("injected panic in worker thread");
            }
        }
    });
    let spike = Spike {
        reaper,
        calls: 0,
        ticks_alive,
        last_report: Instant::now(),
    };
    session.audio_reg_hardware_hook_add(Box::new(AudioHook))?;
    session.plugin_register_add_csurf_inst(Box::new(spike))?;
    Box::leak(Box::new(session));
    Ok(())
}
