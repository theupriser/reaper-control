//! Tauri glue: holds the link, exposes `dispatch` and `current_view` to the UI and tells it when the view changes.

use tauri::{Emitter, Manager, State};

use std::sync::{Arc, Mutex};
use std::time::Duration;

use protocol::{Command, LinkView, SetlistTransferView, Settings, SettingsView};

use crate::app_event::AppEvent;
use crate::app_fault::AppFault;
use crate::command_bus::CommandBus;
use crate::config_location::{
    config_file, diagnostics_directory, legacy_config_file, legacy_setlists_directory,
    log_directory, mirror_directory,
};
use crate::config_store::ConfigStore;
use crate::diagnostics_bundle::DiagnosticsBundle;
use crate::endpoint_location::{endpoint_file, extension_directory, fault_file};
use crate::event_bus::EventBus;
use crate::event_logger::log_event;
use crate::fake_process_check::FakeProcessCheck;
use crate::fault_file_check::FaultFileCheck;
use crate::health_monitor::HealthMonitor;
use crate::intent_dispatcher::IntentDispatcher;
use crate::journal_import::JournalImport;
use crate::legacy_config_import::import_legacy_config;
use crate::link_view_source::LinkViewSource;
use crate::logging::Logging;
use crate::metered_driver::MeteredDriver;
use crate::midi_listener::{MidiListener, device_names};
use crate::midi_router::MidiRouter;
use crate::mirror_keeper::MirrorKeeper;
use crate::notice_for_event::{link_problem, notice_for_event};
use crate::panic_hook::install_panic_hook;
use crate::periodic_thread::PeriodicThread;
use crate::process_check::ProcessCheck;
use crate::setlist_mirror::SetlistMirror;
use crate::setlist_transfer::SetlistTransfer;
use crate::settings_service::SettingsService;
use crate::shutdown_sequence::ShutdownSequence;
use crate::start_link::start_link;
use crate::system_clock::SystemClock;
use crate::system_process_check::SystemProcessCheck;

const VIEW_CHANGED: &str = "link-view";
const NOTICE: &str = "notice";
const LINK_PROBLEM: &str = "link-problem";

#[tauri::command]
fn current_view(link: State<'_, Arc<dyn LinkViewSource>>) -> LinkView {
    link.view()
}

#[tauri::command]
fn current_problem(
    health: State<'_, Arc<HealthMonitor>>,
    fault: State<'_, Arc<AppFault>>,
) -> Option<String> {
    fault.message().or_else(|| link_problem(&health.health()))
}

#[tauri::command]
fn current_settings(settings: State<'_, Arc<SettingsService>>) -> SettingsView {
    settings.view(device_names().unwrap_or_default())
}

#[tauri::command]
fn save_settings(
    service: State<'_, Arc<SettingsService>>,
    settings: Settings,
) -> Result<(), String> {
    service.save(&settings).map_err(|error| error.to_string())
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
            let health = Arc::new(HealthMonitor::new(
                events.clone(),
                clock.clone(),
                processes,
                Arc::new(FaultFileCheck::new(
                    fault_file().ok_or("the home folder is unknown")?,
                )),
            ));
            let mirror =
                SetlistMirror::new(mirror_directory().ok_or("the home folder is unknown")?);
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
            let settings_file = config_file().ok_or("the home folder is unknown")?;
            app.manage(Arc::new(SettingsService::new(
                ConfigStore::new(settings_file),
                config.clone(),
                bus.clone(),
            )));
            app.manage(Arc::new(SetlistTransfer::new(
                mirror,
                legacy_setlists_directory(),
                bus.clone(),
            )));
            if config.midi.enabled {
                let watched = link.clone();
                let intents = Arc::new(IntentDispatcher::new(
                    bus.clone(),
                    events.clone(),
                    move || watched.view(),
                ));
                let router = Arc::new(MidiRouter::new(
                    config.midi.clone(),
                    intents,
                    events.clone(),
                    clock,
                ));
                if let Err(error) =
                    MidiListener::start(router, config.midi.device_name.clone(), events)
                {
                    tracing::error!(%error, "midi thread failed to start");
                }
            }
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
            app.manage(Mutex::new(threads));
            app.manage(health);
            app.manage(bus);
            app.manage(link);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            current_view,
            current_problem,
            current_settings,
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
