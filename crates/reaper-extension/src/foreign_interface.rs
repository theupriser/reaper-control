//! The one module that talks to REAPER and the C runtime. Everything else in the crate is safe.

mod surface;

use std::error::Error;
use std::path::PathBuf;
use std::sync::OnceLock;

use reaper_low::PluginContext;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::ReaperSession;

use crate::fault::Fault;
use crate::log::Log;
use crate::safe_mode_marker::SafeModeMarker;
use crate::start_up::StartUp;
use surface::Surface;

static FAULT: Fault = Fault::new();
static LOG: OnceLock<Log> = OnceLock::new();
static MARKER: OnceLock<SafeModeMarker> = OnceLock::new();

unsafe extern "C" {
    fn atexit(callback: extern "C" fn()) -> i32;
}

/// REAPER does not call `close_no_reset` or drop the surface at a clean quit (ADR-009), but the C
/// runtime runs this at exit. A crash, a kill or SIGTERM never gets here, so the marker stays.
extern "C" fn on_exit() {
    let _ = std::panic::catch_unwind(|| {
        if let Some(marker) = MARKER.get() {
            marker.release();
        }
    });
}

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    match FAULT.guard(|| start(context)) {
        Some(result) => result,
        None => Err("the extension panicked while starting".into()),
    }
}

fn start(context: PluginContext) -> Result<(), Box<dyn Error>> {
    let mut session = ReaperSession::load(context);
    let resource = session.reaper().get_resource_path(|path| path.to_string());
    let directory = PathBuf::from(resource).join("RC2");
    std::fs::create_dir_all(&directory)?;

    let log = LOG.get_or_init(|| Log::new(directory.join("extension.log")));
    FAULT.install_panic_hook(log);

    match SafeModeMarker::claim(&directory.join("running"))? {
        StartUp::SafeMode => {
            log.line("SAFE MODE: REAPER did not shut down cleanly last time; extension disabled");
            session
                .reaper()
                .show_console_msg("RC2: safe mode, extension disabled\n");
            return Ok(());
        }
        StartUp::Normal(marker) => {
            let _ = MARKER.set(marker);
        }
    }
    // SAFETY: registers a plain `extern "C" fn` that only removes a file.
    let registered = unsafe { atexit(on_exit) };
    log.line(&format!("started, atexit registered: {registered}"));

    session.plugin_register_add_csurf_inst(Box::new(Surface::new(&FAULT, log, directory)))?;
    Box::leak(Box::new(session));
    Ok(())
}
