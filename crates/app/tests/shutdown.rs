//! Ending the app in order, and what a panic leaves behind.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use app::AppFault;
use app::PeriodicThread;
use app::ShutdownSequence;
use app::install_panic_hook;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn held<T: Clone>(shared: &Mutex<T>) -> T {
    shared
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
}

#[test]
fn steps_run_in_order_and_a_failing_step_does_not_stop_the_rest() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mut sequence = ShutdownSequence::default();
    for name in ["threads", "link", "log"] {
        let order = order.clone();
        sequence.add(name, move || {
            order
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .push(name);
            assert!(name != "link", "link would not close");
        });
    }
    assert_eq!(sequence.run(), vec!["link"]);
    assert_eq!(held(&order), vec!["threads", "link", "log"]);
}

#[test]
fn a_periodic_thread_works_until_stopped_and_stops_promptly() -> TestResult {
    let count = Arc::new(Mutex::new(0u32));
    let counting = count.clone();
    let thread = PeriodicThread::start("test-periodic", Duration::from_millis(10), move || {
        *counting.lock().unwrap_or_else(|error| error.into_inner()) += 1;
    })
    .ok_or("thread did not start")?;
    let deadline = Instant::now() + Duration::from_secs(5);
    while held(&count) < 3 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    let started = Instant::now();
    assert!(thread.stop());
    assert!(started.elapsed() < Duration::from_secs(2));
    let seen = held(&count);
    assert!(seen >= 3, "worked {seen} times");
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(held(&count), seen);
    Ok(())
}

#[test]
fn a_panic_in_a_worker_thread_is_reported_and_kept_for_the_window() -> TestResult {
    let fault = Arc::new(AppFault::default());
    let recorder = fault.clone();
    let reported = Arc::new(Mutex::new(Vec::<String>::new()));
    let reporting = reported.clone();
    install_panic_hook(move |panic| {
        if !panic.contains("test-worker") {
            return;
        }
        recorder.record(panic);
        reporting
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(panic.to_owned());
    });
    assert!(fault.message().is_none());

    let worker = std::thread::Builder::new()
        .name("test-worker".into())
        .spawn(|| panic!("worker broke"))?;
    assert!(worker.join().is_err());

    let reports = held(&reported);
    let report = reports
        .iter()
        .find(|report| report.contains("worker broke"))
        .ok_or("the panic was not reported")?;
    assert!(report.contains("test-worker") && report.contains("shutdown.rs"));
    let message = fault.message().ok_or("no fault kept")?;
    assert!(message.contains("REAPER keeps playing") && message.contains("worker broke"));
    Ok(())
}
