//! Spike S6: stable identity of regions and markers. The control surface tick polls
//! `<resource>/RC2/s6-cmd` for one command line, runs it on the main thread, deletes the file and
//! logs to `<resource>/RC2/spike-s6.log`. Commands:
//!   dump <label>                      list every region/marker with its GUID
//!   edit <id> <region|marker> <start> <end> <name...>   change position/name (SetProjectMarker4)
//!   insert <start> <end> <name...>    new region (AddProjectMarker2)
//!   delete <id> <region|marker>
//!   save | reload <path> | quit
#![allow(unsafe_code)] // spike only

use std::error::Error;
use std::ffi::{CStr, CString};
use std::fs::OpenOptions;
use std::io::Write;
use std::ptr::null_mut;

use reaper_low::PluginContext;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::{
    CommandId, ControlSurface, MainThreadScope, OpenProjectBehavior, ProjectContext, ProjectRef,
    Reaper, ReaperSession,
};

#[derive(Debug)]
struct Spike {
    reaper: Reaper<MainThreadScope>,
    dir: String,
}

impl Spike {
    fn log(&self, line: &str) {
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("{}/spike-s6.log", self.dir))
        {
            let _ = writeln!(f, "{line}");
        }
    }

    fn project_ptr(&self) -> *mut reaper_low::raw::ReaProject {
        self.reaper
            .enum_projects(ProjectRef::Current, 0)
            .map(|p| p.project.as_ptr())
            .unwrap_or(null_mut())
    }

    fn guid(&self, index: u32) -> String {
        let desc = CString::new(format!("MARKER_GUID:{index}")).unwrap_or_default();
        let mut buf = vec![0 as std::os::raw::c_char; 128];
        let ok = unsafe {
            self.reaper.low().GetSetProjectInfo_String(
                self.project_ptr(),
                desc.as_ptr(),
                buf.as_mut_ptr(),
                false,
            )
        };
        if !ok {
            return "(none)".into();
        }
        unsafe { CStr::from_ptr(buf.as_ptr()) }.to_string_lossy().into_owned()
    }

    fn dump(&self, label: &str) {
        let project = ProjectContext::CurrentProject;
        let counts = self.reaper.count_project_markers(project);
        self.log(&format!(
            "--- {label}: {} regions, {} markers ---",
            counts.region_count, counts.marker_count
        ));
        for index in 0..counts.total_count {
            let line = self
                .reaper
                .enum_project_markers_3(project, index, |item| match item {
                    Some(m) => {
                        let kind = if m.region_end_position.is_some() { "region" } else { "marker" };
                        let end = m.region_end_position.map(|e| e.get()).unwrap_or(0.0);
                        format!(
                            "{index:>2} {kind} id={} guid={} start={:.3} end={:.3} name={:?}",
                            m.id.get(), "{G}", m.position.get(), end, m.name.to_str()
                        )
                    }
                    None => format!("{index:>2} (none)"),
                });
            self.log(&line.replace("{G}", &self.guid(index)));
        }
    }

    fn run_command(&mut self, line: &str) {
        let mut w = line.split_whitespace();
        let cmd = w.next().unwrap_or("");
        let rest: Vec<&str> = w.collect();
        let proj = self.project_ptr();
        let low = self.reaper.low();
        match (cmd, rest.as_slice()) {
            ("dump", _) => self.dump(&rest.join(" ")),
            ("edit", [id, kind, start, end, name @ ..]) => {
                let n = CString::new(name.join(" ")).unwrap_or_default();
                let ok = unsafe {
                    low.SetProjectMarker4(
                        proj,
                        id.parse().unwrap_or(-1),
                        *kind == "region",
                        start.parse().unwrap_or(0.0),
                        end.parse().unwrap_or(0.0),
                        n.as_ptr(),
                        0,
                        0,
                    )
                };
                self.log(&format!("edit {line} -> {ok}"));
            }
            ("insert", [start, end, name @ ..]) => {
                let n = CString::new(name.join(" ")).unwrap_or_default();
                let id = unsafe {
                    low.AddProjectMarker2(
                        proj,
                        true,
                        start.parse().unwrap_or(0.0),
                        end.parse().unwrap_or(0.0),
                        n.as_ptr(),
                        -1,
                        0,
                    )
                };
                self.log(&format!("insert {line} -> id {id}"));
            }
            ("delete", [id, kind]) => {
                let ok = unsafe {
                    low.DeleteProjectMarker(proj, id.parse().unwrap_or(-1), *kind == "region")
                };
                self.log(&format!("delete {line} -> {ok}"));
            }
            ("save", _) => {
                self.reaper.main_on_command_ex(CommandId::new(40026), 0, ProjectContext::CurrentProject);
                self.log("save requested (action 40026)");
            }
            ("reload", [path]) => {
                let mut behavior = OpenProjectBehavior::default();
                behavior.prompt = false;
                self.reaper
                    .main_open_project(camino::Utf8Path::new(path), behavior);
                self.log(&format!("reload requested: {path}"));
            }
            _ => self.log(&format!("unknown command: {line}")),
        }
    }
}

impl ControlSurface for Spike {
    fn run(&mut self) {
        let path = format!("{}/s6-cmd", self.dir);
        if let Ok(text) = std::fs::read_to_string(&path) {
            let _ = std::fs::remove_file(&path);
            for l in text.lines().filter(|l| !l.trim().is_empty()) {
                self.run_command(l.trim());
            }
        }
    }
}

#[reaper_extension_plugin]
fn plugin_main(context: PluginContext) -> Result<(), Box<dyn Error>> {
    let mut session = ReaperSession::load(context);
    let reaper = session.reaper().clone();
    let resource = reaper.get_resource_path(|p| p.to_string());
    let dir = format!("{resource}/RC2");
    let _ = std::fs::create_dir_all(&dir);
    let spike = Spike { reaper, dir };
    spike.log(&format!("S6 loaded, REAPER {}", spike.reaper.get_app_version()));
    session.plugin_register_add_csurf_inst(Box::new(spike))?;
    Box::leak(Box::new(session));
    Ok(())
}
