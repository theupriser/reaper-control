use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{Autostarter, Launcher};

#[derive(Clone, Default)]
struct RecordingLauncher {
    started: Rc<RefCell<Vec<PathBuf>>>,
    fails: bool,
}

impl Launcher for RecordingLauncher {
    fn launch(&mut self, application: &Path) -> Result<(), String> {
        if self.fails {
            return Err("no such program".into());
        }
        self.started.borrow_mut().push(application.to_path_buf());
        Ok(())
    }
}

struct Scene {
    autostarter: Autostarter,
    started: Rc<RefCell<Vec<PathBuf>>>,
    directory: PathBuf,
}

impl Drop for Scene {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

static SCENES: AtomicUsize = AtomicUsize::new(0);

fn scene(location: Option<&str>, fails: bool) -> Result<Scene, std::io::Error> {
    let directory = std::env::temp_dir().join(format!(
        "rc2-autostart-{}-{}",
        std::process::id(),
        SCENES.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory)?;
    let file = directory.join("application");
    if let Some(location) = location {
        std::fs::write(&file, format!("{location}\n"))?;
    }
    let launcher = RecordingLauncher {
        fails,
        ..RecordingLauncher::default()
    };
    let started = Rc::clone(&launcher.started);
    Ok(Scene {
        autostarter: Autostarter::new(file, Box::new(launcher)),
        started,
        directory,
    })
}

fn run(
    scene: &mut Scene,
    project: &str,
    autostart: bool,
    clients: usize,
    seconds: u32,
) -> Vec<String> {
    (0..=seconds)
        .filter_map(|second| {
            scene
                .autostarter
                .observe(project, autostart, clients, f64::from(second))
        })
        .collect()
}

#[test]
fn a_project_that_asks_starts_the_app_once_after_the_grace_time() -> Result<(), std::io::Error> {
    let mut scene = scene(Some("/Applications/Reaper Control.app"), false)?;
    assert!(run(&mut scene, "p1", true, 0, 1).is_empty());
    assert!(scene.started.borrow().is_empty());
    let lines = run(&mut scene, "p1", true, 0, 60);
    assert_eq!(lines.len(), 1);
    assert_eq!(
        scene.started.borrow().as_slice(),
        [PathBuf::from("/Applications/Reaper Control.app")]
    );
    Ok(())
}

#[test]
fn nothing_starts_when_the_project_does_not_ask() -> Result<(), std::io::Error> {
    let mut scene = scene(Some("/app"), false)?;
    assert!(run(&mut scene, "p1", false, 0, 60).is_empty());
    assert!(scene.started.borrow().is_empty());
    Ok(())
}

#[test]
fn nothing_starts_when_the_app_connects_in_time() -> Result<(), std::io::Error> {
    let mut scene = scene(Some("/app"), false)?;
    run(&mut scene, "p1", true, 0, 1);
    run(&mut scene, "p1", true, 1, 60);
    assert!(scene.started.borrow().is_empty());
    Ok(())
}

#[test]
fn an_app_that_is_closed_later_is_not_started_again() -> Result<(), std::io::Error> {
    let mut scene = scene(Some("/app"), false)?;
    run(&mut scene, "p1", true, 1, 2);
    run(&mut scene, "p1", true, 0, 60);
    assert!(scene.started.borrow().is_empty());
    Ok(())
}

#[test]
fn a_project_without_an_id_yet_waits() -> Result<(), std::io::Error> {
    let mut scene = scene(Some("/app"), false)?;
    assert!(run(&mut scene, "", true, 0, 60).is_empty());
    assert!(scene.started.borrow().is_empty());
    Ok(())
}

#[test]
fn each_project_that_asks_is_looked_at_on_its_own() -> Result<(), std::io::Error> {
    let mut scene = scene(Some("/app"), false)?;
    run(&mut scene, "p1", true, 0, 60);
    run(&mut scene, "p2", false, 0, 60);
    run(&mut scene, "p3", true, 0, 60);
    assert_eq!(scene.started.borrow().len(), 2);
    Ok(())
}

#[test]
fn an_unknown_location_is_said_and_nothing_is_started() -> Result<(), std::io::Error> {
    let mut scene = scene(None, false)?;
    let lines = run(&mut scene, "p1", true, 0, 60);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("open the installed app once"));
    assert!(scene.started.borrow().is_empty());
    Ok(())
}

#[test]
fn a_failed_start_is_logged_and_not_tried_again() -> Result<(), std::io::Error> {
    let mut scene = scene(Some("/app"), true)?;
    let lines = run(&mut scene, "p1", true, 0, 60);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("could not start"));
    Ok(())
}
