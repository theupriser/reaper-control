//! The one module that talks to REAPER and the C runtime. Everything else in the crate is safe.

#[cfg(windows)]
mod dll_main;
mod exit_file;
mod missing_functions;
#[cfg(feature = "probe")]
mod probe;
mod reaper_rs_adapter;
mod required_functions;
mod surface;

use std::error::Error;
use std::path::PathBuf;
use std::sync::OnceLock;

use reaper_low::PluginContext;
use reaper_medium::ReaperSession;

use crate::fault::Fault;
use crate::fault_file::FaultFile;
use crate::log::Log;
use crate::safe_mode_marker::SafeModeMarker;
use crate::start_up::StartUp;
use exit_file::ExitFile;
use missing_functions::MissingFunctions;
use reaper_rs_adapter::ReaperRsAdapter;
use surface::Surface;

static FAULT: Fault = Fault::new();
static LOG: OnceLock<Log> = OnceLock::new();
static FAULTS: OnceLock<FaultFile> = OnceLock::new();
static EXIT_FILE: OnceLock<ExitFile> = OnceLock::new();

unsafe extern "C" {
    fn atexit(callback: extern "C" fn()) -> i32;
}

/// REAPER does not call `close_no_reset` or drop the surface at a clean quit (ADR-009), but the C
/// runtime runs this at exit. A crash, a kill or SIGTERM never gets here, so the marker stays.
extern "C" fn on_exit() {
    if let Some(file) = EXIT_FILE.get() {
        file.remove();
    }
}

reaper_low::swell_dll_main!();

/// The entry point REAPER calls at startup; what `#[reaper_extension_plugin]` would generate,
/// without its `DllMain` (see `dll_main`).
#[unsafe(no_mangle)]
unsafe extern "C" fn ReaperPluginEntry(
    h_instance: reaper_low::raw::HINSTANCE,
    rec: *mut reaper_low::raw::reaper_plugin_info_t,
) -> std::os::raw::c_int {
    let static_context = reaper_low::static_plugin_context();
    // SAFETY: REAPER passes the handle and the plugin info record it owns, as the macro assumes.
    unsafe { reaper_low::bootstrap_extension_plugin(h_instance, rec, static_context, plugin_main) }
}

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
    let faults = FAULTS.get_or_init(|| FaultFile::new(directory.join("faulted")));
    FAULT.install_panic_hook(log, faults);

    if let Err(missing) = MissingFunctions::check(context) {
        log.line(&format!("REFUSING TO START: {missing}"));
        let _ = faults.report(&format!("this REAPER is too old: {missing}"));
        session.reaper().show_console_msg(format!(
            "RC2: extension disabled, this REAPER is too old: {missing}\n"
        ));
        return Ok(());
    }

    match SafeModeMarker::claim(&directory.join("running"))? {
        StartUp::SafeMode => {
            log.line("SAFE MODE: REAPER did not shut down cleanly last time; extension disabled");
            let _ = faults.report("REAPER did not shut down cleanly last time (safe mode)");
            session
                .reaper()
                .show_console_msg("RC2: safe mode, extension disabled\n");
            return Ok(());
        }
        StartUp::Normal(marker) => {
            faults.clear();
            if let Some(file) = ExitFile::new(marker.path()) {
                let _ = EXIT_FILE.set(file);
            }
        }
    }
    // SAFETY: registers a plain `extern "C" fn` that only removes a file.
    let registered = unsafe { atexit(on_exit) };
    log.line(&format!(
        "started, version {}, atexit registered: {registered}",
        env!("CARGO_PKG_VERSION")
    ));

    session.plugin_register_add_csurf_inst(Box::new(Surface::new(
        &FAULT,
        log,
        faults,
        directory,
        ReaperRsAdapter::new(session.reaper().clone()),
    )))?;
    Box::leak(Box::new(session));
    Ok(())
}
