//! Tauri glue: holds the link, exposes `dispatch` and `current_view` to the UI and tells it when the view changes.

use tauri::{Emitter, Manager, State};

use std::sync::{Arc, Mutex};
use std::time::Duration;

use protocol::{
    ChecklistView, Command, InstallationView, LinkProblem, LinkStatus, LinkView, MarkerReading,
    SetlistTransferView, Settings, SettingsView, SystemStats,
};

use crate::health::FaultFileCheck;
use crate::health::HealthMonitor;
use crate::link::LinkViewSource;
use crate::marker::read_marker_name;
use crate::mirror::MirrorKeeper;
use crate::mirror::MirrorRepository;
use crate::mirror::SetlistMirror;
use crate::mirror::SetlistTransfer;
use crate::start::start_link;
use crate::start::{record_application_location, start_installation};
use crate::system::DiagnosticsBundle;
use crate::system::Logging;
use crate::system::PeriodicThread;
use crate::system::ShutdownSequence;
use crate::system::SysinfoStatsSource;
use crate::system::SystemStatsService;
use crate::system::install_panic_hook;
use app_core::AppEvent;
use app_core::AppFault;
use app_core::CommandBus;
use app_core::ConfigRepository;
use app_core::ConfigStore;
use app_core::EventBus;
use app_core::IntentDispatcher;
use app_core::MeteredDriver;
use app_core::SettingsService;
use app_core::SystemClock;
use app_core::log_event;
use app_core::{
    config_file, diagnostics_directory, legacy_config_file, legacy_setlists_directory,
    log_directory, mirror_directory,
};
use app_core::{link_problem, notice_for_event};
use installer::FakeProcessCheck;
use installer::InstallationService;
use installer::ProcessCheck;
use installer::SystemProcessCheck;
use installer::check_list;
use installer::{application_file, endpoint_file, extension_directory, fault_file, reset_fault};
use legacy_import::JournalImport;
use legacy_import::import_legacy_config;
use midi::MidiSource;
use midi::MidiSwitch;
use midi::MidirAnchor;
use midi::MidirSource;

const VIEW_CHANGED: &str = "link-view";
const NOTICE: &str = "notice";
const LINK_PROBLEM: &str = "link-problem";
/// The version of the extension that ships with this app (kept equal to the extension's own
/// version by an architecture test).
const BUNDLED_EXTENSION_VERSION: &str = "1.0.7";
const UNSUPPORTED_SYSTEM: &str = "the extension is not built for this system";

#[tauri::command]
fn bundled_extension_version() -> &'static str {
    BUNDLED_EXTENSION_VERSION
}

#[tauri::command]
fn current_view(link: State<'_, Arc<dyn LinkViewSource>>) -> LinkView {
    link.view()
}

#[tauri::command]
fn current_problem(
    health: State<'_, Arc<HealthMonitor>>,
    fault: State<'_, Arc<AppFault>>,
) -> Option<LinkProblem> {
    fault
        .message()
        .map(|message| LinkProblem {
            message,
            extension_outdated: false,
            extension_faulted: false,
        })
        .or_else(|| link_problem(&health.health()))
}

#[tauri::command]
fn reenable_extension() -> Result<(), String> {
    let directory = extension_directory().ok_or("the home folder is unknown")?;
    reset_fault(&directory).map_err(|error| error.to_string())
}

#[tauri::command]
fn current_system_stats(stats: State<'_, Arc<SystemStatsService>>) -> SystemStats {
    stats.refresh();
    stats.latest()
}

#[tauri::command]
fn marker_reading(name: String) -> MarkerReading {
    read_marker_name(&name)
}

#[tauri::command]
fn current_settings(settings: State<'_, Arc<SettingsService>>) -> SettingsView {
    settings.view(MidirSource.device_names().unwrap_or_default())
}

#[tauri::command]
fn save_settings(
    service: State<'_, Arc<SettingsService>>,
    settings: Settings,
) -> Result<(), String> {
    service.save(&settings).map_err(|error| error.to_string())
}

#[tauri::command]
fn current_installation(
    installation: State<'_, Option<Arc<InstallationService>>>,
    link: State<'_, Arc<dyn LinkViewSource>>,
) -> Result<InstallationView, String> {
    let service = installation.as_ref().ok_or(UNSUPPORTED_SYSTEM)?;
    Ok(service.view(is_connected(&link.view().status)))
}

#[tauri::command]
fn install_extension(
    installation: State<'_, Option<Arc<InstallationService>>>,
    link: State<'_, Arc<dyn LinkViewSource>>,
) -> Result<InstallationView, String> {
    let service = installation.as_ref().ok_or(UNSUPPORTED_SYSTEM)?;
    service.install(is_connected(&link.view().status))
}

#[tauri::command]
fn current_checklist(
    installation: State<'_, Option<Arc<InstallationService>>>,
    link: State<'_, Arc<dyn LinkViewSource>>,
    settings: State<'_, Arc<SettingsService>>,
) -> ChecklistView {
    let view = link.view();
    let installed = installation
        .as_ref()
        .map(|service| service.view(is_connected(&view.status)));
    let found = settings.view(MidirSource.device_names().unwrap_or_default());
    check_list(&view, &found.settings, &found.devices, installed.as_ref())
}

fn is_connected(status: &LinkStatus) -> bool {
    matches!(status, LinkStatus::Connected { .. })
}

#[tauri::command]
fn current_transfer(
    transfer: State<'_, Arc<SetlistTransfer>>,
    link: State<'_, Arc<dyn LinkViewSource>>,
) -> SetlistTransferView {
    transfer.view(&link.view())
}

#[tauri::command]
fn restore_setlists(
    transfer: State<'_, Arc<SetlistTransfer>>,
    link: State<'_, Arc<dyn LinkViewSource>>,
) -> Result<usize, String> {
    transfer.restore(&link.view())
}

#[tauri::command]
fn import_setlists(
    transfer: State<'_, Arc<SetlistTransfer>>,
    link: State<'_, Arc<dyn LinkViewSource>>,
    ids: Vec<String>,
) -> Result<usize, String> {
    transfer.import(&link.view(), &ids)
}

#[tauri::command]
fn export_diagnostics(
    bundle: State<'_, Arc<DiagnosticsBundle>>,
    journal: State<'_, Arc<JournalImport>>,
    link: State<'_, Arc<dyn LinkViewSource>>,
) -> Result<String, String> {
    journal.import();
    let directory = diagnostics_directory().ok_or("the home folder is unknown")?;
    let status = format!("{:?}", link.view().status);
    bundle
        .export(&directory, &status)
        .map(|path| path.display().to_string())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn dispatch(bus: State<'_, Arc<CommandBus>>, command: Command) -> Result<(), String> {
    bus.dispatch(command).map_err(|error| error.to_string())
}

/// Starts the window and blocks until it closes.
pub fn run() {
    let level = config_file()
        .and_then(|file| ConfigStore::new(file).load().ok())
        .map_or_else(|| "info".to_owned(), |config| config.log.level);
    let logging = Logging::start(log_directory(), &level);
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(
            |app, _arguments, _directory| {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            },
        ))
        .setup(|app| {
            let handle = app.handle().clone();
            let fault = Arc::new(AppFault::default());
            let reporter = handle.clone();
            let recorder = fault.clone();
            install_panic_hook(move |panic| {
                let _ = reporter.emit(LINK_PROBLEM, Some(recorder.record(panic)));
            });
            app.manage(fault);
            let file = endpoint_file().ok_or("the home folder is unknown")?;
            if !tauri::is_dev()
                && let Some(location) = application_file()
                && let Ok(executable) = std::env::current_exe()
                && let Err(error) = record_application_location(&location, &executable)
            {
                tracing::warn!(%error, "could not tell the extension where the app is");
            }
            let events = Arc::new(EventBus::default());
            events.subscribe(log_event);
            let journal = Arc::new(JournalImport::new(
                extension_directory()
                    .ok_or("the home folder is unknown")?
                    .join("journal.log"),
            ));
            journal.import();
            let mut threads = Vec::new();
            let importing = journal.clone();
            threads.extend(PeriodicThread::start(
                "journal-import",
                Duration::from_secs(2),
                move || {
                    importing.import();
                },
            ));
            app.manage(journal);
            app.manage(Arc::new(DiagnosticsBundle::new(
                log_directory(),
                extension_directory(),
                config_file(),
            )));
            let notices = handle.clone();
            events.subscribe(move |event| {
                if let Some(notice) = notice_for_event(event) {
                    let _ = notices.emit(NOTICE, notice);
                }
                if let AppEvent::LinkHealthChanged { health } = event {
                    let _ = notices.emit(LINK_PROBLEM, link_problem(health));
                }
            });
            if let Some((file, legacy)) = config_file().zip(legacy_config_file()) {
                let _ = import_legacy_config(&ConfigStore::new(file), &legacy);
            }
            let config = config_file()
                .map(|file| ConfigStore::new(file).load())
                .transpose()
                .unwrap_or_else(|error| {
                    tracing::warn!(%error, "using the default settings");
                    None
                })
                .unwrap_or_default();
            let clock = Arc::new(SystemClock::new());
            let simulated = std::env::var_os("RC2_SIMULATOR").is_some();
            let processes: Arc<dyn ProcessCheck> = if simulated {
                let simulated_reaper = FakeProcessCheck::default();
                simulated_reaper.set_running(true);
                Arc::new(simulated_reaper)
            } else {
                Arc::new(SystemProcessCheck)
            };
            app.manage(start_installation(processes.clone()).map(Arc::new));
            let health = Arc::new(HealthMonitor::new(
                events.clone(),
                clock.clone(),
                processes,
                Arc::new(FaultFileCheck::new(
                    fault_file().ok_or("the home folder is unknown")?,
                )),
            ));
            let mirror: Arc<dyn MirrorRepository> = Arc::new(SetlistMirror::new(
                mirror_directory().ok_or("the home folder is unknown")?,
            ));
            let keeper = MirrorKeeper::new(mirror.clone());
            let (link_driver, link) = start_link(
                simulated,
                file,
                events.clone(),
                health.clone(),
                move |view| {
                    keeper.observe(&view);
                    let _ = handle.emit(VIEW_CHANGED, view);
                },
            );
            let driver = Arc::new(MeteredDriver::new(link_driver, clock.clone(), &events));
            let bus = Arc::new(CommandBus::new(
                driver,
                events.clone(),
                clock.clone(),
                config.queue.settings(),
            ));
            let watched = link.clone();
            let intents = Arc::new(IntentDispatcher::new(
                bus.clone(),
                events.clone(),
                move || watched.view(),
            ));
            // Before the MIDI thread starts: CoreMIDI wants its first client on the main thread.
            app.manage(MidirAnchor::new());
            let midi = Arc::new(MidiSwitch::new(
                Arc::new(MidirSource),
                intents,
                events,
                clock,
            ));
            midi.apply(&config.midi);
            let settings_file = config_file().ok_or("the home folder is unknown")?;
            app.manage(Arc::new(SettingsService::new(
                Arc::new(ConfigStore::new(settings_file)),
                config.clone(),
                bus.clone(),
                Arc::new(move |changed| midi.apply(changed)),
            )));
            app.manage(Arc::new(SetlistTransfer::new(
                mirror,
                legacy_setlists_directory(),
                bus.clone(),
            )));
            let watched = Arc::clone(&bus);
            threads.extend(PeriodicThread::start(
                "command-timeouts",
                Duration::from_millis(500),
                move || watched.expire(),
            ));
            let ticking = health.clone();
            threads.extend(PeriodicThread::start(
                "link-health",
                Duration::from_secs(1),
                move || ticking.tick(),
            ));
            let stats = Arc::new(SystemStatsService::new(SysinfoStatsSource::default()));
            app.manage(stats);
            app.manage(Mutex::new(threads));
            app.manage(health);
            app.manage(bus);
            app.manage(link);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bundled_extension_version,
            current_view,
            current_problem,
            reenable_extension,
            current_system_stats,
            marker_reading,
            current_settings,
            current_installation,
            install_extension,
            current_checklist,
            save_settings,
            current_transfer,
            restore_setlists,
            import_setlists,
            export_diagnostics,
            dispatch
        ])
        .build(tauri::generate_context!());
    let app = match result {
        Ok(app) => app,
        Err(error) => {
            tracing::error!(%error, "reaper control failed to start");
            drop(logging);
            std::process::exit(1);
        }
    };
    let mut logging = Some(logging);
    app.run(move |handle, event| {
        if let tauri::RunEvent::Exit = event {
            let mut sequence = ShutdownSequence::default();
            if let Some(threads) = handle.try_state::<Mutex<Vec<PeriodicThread>>>() {
                let threads = threads
                    .lock()
                    .map(|mut held| std::mem::take(&mut *held))
                    .unwrap_or_default();
                sequence.add("background threads", move || {
                    for thread in threads {
                        thread.stop();
                    }
                });
            }
            let closing = logging.take();
            sequence.add("log", move || drop(closing));
            let _ = sequence.run();
        }
    });
}
