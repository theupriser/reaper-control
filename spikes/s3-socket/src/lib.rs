//! Spike S3: loopback socket push from a REAPER extension. The main-thread tick only does
//! `try_send` into bounded channels; accept, read, write and fan-out run on other threads.
//! A slow or dead client is dropped, never waited for. State lines are padded to 4 KB so a
//! client that stops reading fills its buffers within seconds.
#![allow(unsafe_code)] // spike only

use std::error::Error;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError, TrySendError, sync_channel};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use reaper_low::PluginContext;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::{ControlSurface, MainThreadScope, ProjectContext, Reaper, ReaperSession};

const CLIENT_QUEUE: usize = 32;
const STATE_BYTES: usize = 4096;
const REPORT_EVERY: Duration = Duration::from_secs(3);

#[derive(Default, Debug)]
struct Stats {
    clients: AtomicU64,
    dropped_slow: AtomicU64,
    auth_failed: AtomicU64,
    main_full: AtomicU64,
}

enum Msg {
    Line(Arc<str>),
    NewClient(SyncSender<Arc<str>>),
}

fn now_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

fn log(path: &str, line: &str) {
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{line}");
    }
}

/// Fans every line out to all clients with `try_send`; a full or closed queue removes the client.
fn broadcaster(rx: Receiver<Msg>, stats: Arc<Stats>, log_path: String) {
    let mut clients: Vec<SyncSender<Arc<str>>> = Vec::new();
    while let Ok(msg) = rx.recv() {
        match msg {
            Msg::NewClient(tx) => clients.push(tx),
            Msg::Line(line) => clients.retain(|c| match c.try_send(line.clone()) {
                Ok(()) => true,
                Err(TrySendError::Full(_)) => {
                    stats.dropped_slow.fetch_add(1, Relaxed);
                    log(&log_path, "client dropped: queue full (slow reader)");
                    false
                }
                Err(TrySendError::Disconnected(_)) => false,
            }),
        }
        stats.clients.store(clients.len() as u64, Relaxed);
    }
}

fn serve(
    stream: TcpStream,
    token: &str,
    out: SyncSender<Msg>,
    inbound: SyncSender<String>,
    stats: &Stats,
    log_path: &str,
) {
    let Ok(read_half) = stream.try_clone() else {
        return;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut reader = BufReader::new(read_half);
    let mut hello = String::new();
    let ok = reader.read_line(&mut hello).is_ok() && hello.trim() == format!("HELLO {token}");
    if !ok {
        stats.auth_failed.fetch_add(1, Relaxed);
        log(log_path, "client rejected: bad or missing token");
        return;
    }
    let mut writer = stream;
    let _ = writer.set_read_timeout(None);
    let _ = writer.set_write_timeout(Some(Duration::from_secs(1)));
    if writer.write_all(b"OK\n").is_err() {
        return;
    }
    let (tx, rx) = sync_channel::<Arc<str>>(CLIENT_QUEUE);
    if out.try_send(Msg::NewClient(tx)).is_err() {
        return;
    }
    let lp = log_path.to_string();
    thread::spawn(move || {
        while let Ok(line) = rx.recv() {
            if writer.write_all(line.as_bytes()).is_err() {
                log(&lp, "client writer stopped: write error or 1 s write timeout");
                break;
            }
        }
        let _ = writer.shutdown(std::net::Shutdown::Both);
    });
    let _ = reader.get_ref().set_read_timeout(None);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let _ = inbound.try_send(line.trim().to_string());
            }
        }
    }
}

#[derive(Debug)]
struct Spike {
    reaper: Reaper<MainThreadScope>,
    log_path: String,
    out: SyncSender<Msg>,
    inbound: Receiver<String>,
    stats: Arc<Stats>,
    seq: u64,
    last_tick: Instant,
    last_report: Instant,
    window: (u64, f64, f64, f64), // ticks, min, max, sum (ms)
    padding: String,
}

impl Spike {
    fn send(&self, line: String) {
        if let Err(TrySendError::Full(_)) = self.out.try_send(Msg::Line(line.into())) {
            self.stats.main_full.fetch_add(1, Relaxed);
        }
    }

    fn report(&mut self) {
        let (n, min, max, sum) = self.window;
        let avg = if n > 0 { sum / n as f64 } else { 0.0 };
        let s = &self.stats;
        let line = format!(
            "ticks={n} dt_ms[min/avg/max]={min:.1}/{avg:.1}/{max:.1} clients={} dropped_slow={} auth_failed={} main_queue_full={}",
            s.clients.load(Relaxed),
            s.dropped_slow.load(Relaxed),
            s.auth_failed.load(Relaxed),
            s.main_full.load(Relaxed)
        );
        log(&self.log_path, &line);
        self.window = (0, f64::MAX, 0.0, 0.0);
    }
}

impl ControlSurface for Spike {
    fn run(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f64() * 1000.0;
        self.last_tick = now;
        let w = &mut self.window;
        *w = (w.0 + 1, w.1.min(dt), w.2.max(dt), w.3 + dt);

        let project = ProjectContext::CurrentProject;
        let state = self.reaper.get_play_state_ex(project);
        let pos = self.reaper.get_play_position_2_ex(project).get();
        self.seq += 1;
        let head = format!(
            "S seq={} us={} playing={} pos={pos:.3} pad=",
            self.seq,
            now_us(),
            state.is_playing
        );
        let fill = STATE_BYTES.saturating_sub(head.len() + 1);
        self.send(format!("{head}{}\n", &self.padding[..fill.min(self.padding.len())]));

        loop {
            match self.inbound.try_recv() {
                Ok(cmd) => {
                    if let Some(id) = cmd.strip_prefix("PING ") {
                        self.send(format!("PONG {id} seq={} us={}\n", self.seq, now_us()));
                    }
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        if now.duration_since(self.last_report) >= REPORT_EVERY {
            self.last_report = now;
            self.report();
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
    let log_path = format!("{dir}/spike-s3.log");

    let token = format!("{:x}", now_us() ^ (std::process::id() as u64) << 20);
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    std::fs::write(format!("{dir}/s3-endpoint.txt"), format!("{port} {token}"))?;
    log(&log_path, &format!("listening on 127.0.0.1:{port}"));
    reaper.show_console_msg(format!("RC2 S3: listening on 127.0.0.1:{port}\n"));

    let stats = Arc::new(Stats::default());
    let (out_tx, out_rx) = sync_channel::<Msg>(256);
    let (in_tx, in_rx) = sync_channel::<String>(256);
    {
        let (stats, lp) = (stats.clone(), log_path.clone());
        thread::spawn(move || broadcaster(out_rx, stats, lp));
    }
    {
        let (stats, lp, out) = (stats.clone(), log_path.clone(), out_tx.clone());
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let (token, out, inbound, stats, lp) =
                    (token.clone(), out.clone(), in_tx.clone(), stats.clone(), lp.clone());
                thread::spawn(move || serve(stream, &token, out, inbound, &stats, &lp));
            }
        });
    }
    let spike = Spike {
        reaper,
        log_path,
        out: out_tx,
        inbound: in_rx,
        stats,
        seq: 0,
        last_tick: Instant::now(),
        last_report: Instant::now(),
        window: (0, f64::MAX, 0.0, 0.0),
        padding: "x".repeat(STATE_BYTES),
    };
    session.plugin_register_add_csurf_inst(Box::new(spike))?;
    Box::leak(Box::new(session));
    Ok(())
}
