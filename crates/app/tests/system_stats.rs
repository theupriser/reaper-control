//! The machine stats: mapped from the source, kept, and really readable.

use std::time::Duration;

use app::fake_stats_source::FakeStatsSource;
use app::periodic_thread::PeriodicThread;
use app::sysinfo_stats_source::SysinfoStatsSource;
use app::system_stats_service::SystemStatsService;
use protocol::SystemStats;
use std::sync::Arc;

#[test]
fn the_service_starts_empty_and_keeps_the_last_reading() {
    let stats = SystemStats {
        machine_cpu_percent: 12.5,
        memory_used_megabytes: 4096,
        memory_total_megabytes: 16384,
        app_memory_megabytes: 80,
        reaper_memory_megabytes: None,
    };
    let service = SystemStatsService::new(FakeStatsSource::new(stats.clone()));
    assert_eq!(service.latest(), SystemStats::default());
    service.refresh();
    assert_eq!(service.latest(), stats);
    assert_eq!(service.latest().reaper_memory_megabytes, None);
}

#[test]
fn a_periodic_thread_refreshes_the_service() {
    let stats = SystemStats {
        memory_total_megabytes: 1,
        ..SystemStats::default()
    };
    let service = Arc::new(SystemStatsService::new(FakeStatsSource::new(stats)));
    let refreshing = service.clone();
    let thread = PeriodicThread::start("stats-test", Duration::from_millis(20), move || {
        refreshing.refresh()
    });
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(service.latest().memory_total_megabytes, 1);
    assert!(thread.map(PeriodicThread::stop).unwrap_or(false));
}

#[test]
fn the_real_source_sees_this_machine_and_this_process() {
    let service = SystemStatsService::new(SysinfoStatsSource::default());
    service.refresh();
    std::thread::sleep(Duration::from_millis(300));
    service.refresh();
    let stats = service.latest();
    println!("{stats:?}");
    assert!(stats.memory_total_megabytes > 0);
    assert!(stats.memory_used_megabytes > 0);
    assert!(stats.memory_used_megabytes <= stats.memory_total_megabytes);
    assert!(stats.app_memory_megabytes > 0);
    assert!((0.0..=100.0).contains(&stats.machine_cpu_percent));
}
