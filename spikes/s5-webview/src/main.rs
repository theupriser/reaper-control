//! Spike S5: push fake performance state at 30 Hz into a webview and let the page measure itself.
//! Throwaway code. Mode (`light` or `heavy`) and duration come from the environment.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tauri::{Emitter, Manager};

#[tauri::command]
fn config() -> serde_json::Value {
    let mode = std::env::var("S5_MODE").unwrap_or_else(|_| "light".into());
    let seconds: u64 = std::env::var("S5_SECONDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    serde_json::json!({ "mode": mode, "seconds": seconds })
}

#[tauri::command]
fn report(app: tauri::AppHandle, stats: serde_json::Value) {
    println!("S5 RESULT {stats}");
    app.exit(0);
}

fn micros_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_micros() as u64)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![config, report])
        .setup(|app| {
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let start = Instant::now();
                let mut seq: u64 = 0;
                loop {
                    let next = start + Duration::from_micros(seq * 1_000_000 / 30);
                    std::thread::sleep(next.saturating_duration_since(Instant::now()));
                    let position = (seq as f64 / 30.0) % 240.0;
                    let payload = serde_json::json!({
                        "seq": seq, "sent": micros_now(),
                        "position": position, "length": 240.0,
                        "song": (seq / 900) % 12,
                    });
                    if handle.emit("state", payload).is_err() {
                        break;
                    }
                    seq += 1;
                }
            });
            let _ = app.get_webview_window("main");
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| eprintln!("S5 failed: {e}"));
}
