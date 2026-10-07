//! Spike S7b: API drift. At load, asks REAPER for every function the extension would use
//! and logs the version plus which ones are missing. A missing function is a null pointer
//! from `GetFunc`; reaper-rs methods on it would panic, so the real extension must check
//! first and refuse to start (or degrade) with a clear message.
#![allow(unsafe_code)] // spike only
use std::error::Error;
use std::ffi::CString;
use std::io::Write;

use reaper_low::PluginContext;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::ReaperSession;

const REQUIRED: &[&str] = &[
    "GetPlayStateEx",
    "GetPlayPosition2Ex",
    "GetPlayPositionEx",
    "SetEditCurPos2",
    "GetCursorPositionEx",
    "OnPlayButtonEx",
    "OnPauseButtonEx",
    "OnStopButtonEx",
    "Main_OnCommandEx",
    "EnumProjectMarkers3",
    "GetProjectStateChangeCount",
    "SetProjExtState",
    "GetProjExtState",
    "EnumProjExtState",
    "TimeMap_GetDividedBpmAtTime",
    "Master_GetTempo",
    "GetResourcePath",
    "GetAppVersion",
    "ShowConsoleMsg",
    "plugin_register",
    "MarkProjectDirty",
];

const OPTIONAL: &[&str] = &["CF_GetSWSVersion", "NoSuchFunctionForControl"];

fn available(ctx: &PluginContext, name: &str) -> bool {
    let Ok(c) = CString::new(name) else {
        return false;
    };
    !unsafe { ctx.GetFunc(c.as_ptr()) }.is_null()
}

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    let ctx = context;
    let session = ReaperSession::load(context);
    let reaper = session.reaper().clone();
    let version = reaper.get_app_version().to_string();
    let resource = reaper.get_resource_path(|p| p.to_string());
    let dir = format!("{resource}/RC2");
    std::fs::create_dir_all(&dir)?;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{dir}/spike-s7b.log"))?;
    writeln!(f, "REAPER version string: {version}")?;
    let missing: Vec<_> = REQUIRED.iter().filter(|n| !available(&ctx, n)).collect();
    for n in REQUIRED {
        writeln!(
            f,
            "required {n}: {}",
            if available(&ctx, n) {
                "present"
            } else {
                "MISSING"
            }
        )?;
    }
    for n in OPTIONAL {
        writeln!(
            f,
            "optional {n}: {}",
            if available(&ctx, n) {
                "present"
            } else {
                "absent"
            }
        )?;
    }
    if missing.is_empty() {
        writeln!(
            f,
            "verdict: all {} required functions present",
            REQUIRED.len()
        )?;
    } else {
        writeln!(f, "verdict: REFUSE TO START, missing {missing:?}")?;
    }
    Box::leak(Box::new(session));
    Ok(())
}
