//! Spike S3 test client. Usage: s3-client <endpoint-file> <watch|ping|slow|churn|badtoken> [n]
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn now_us() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_micros() as u64).unwrap_or(0)
}

fn field(line: &str, key: &str) -> Option<u64> {
    line.split_whitespace().find_map(|p| p.strip_prefix(key)?.strip_prefix('=')?.parse().ok())
}

fn connect(endpoint: &str, token_override: Option<&str>) -> TcpStream {
    let text = std::fs::read_to_string(endpoint).expect("endpoint file");
    let (port, token) = text.trim().split_once(' ').expect("port token");
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).expect("connect");
    s.write_all(format!("HELLO {}\n", token_override.unwrap_or(token)).as_bytes()).unwrap();
    s
}

fn stats(name: &str, mut v: Vec<f64>) {
    if v.is_empty() {
        println!("{name}: no samples");
        return;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    println!(
        "{name}: n={} min={:.2} mean={:.2} p99={:.2} max={:.2} ms",
        v.len(), v[0], mean, v[(v.len() * 99 / 100).min(v.len() - 1)], v[v.len() - 1]
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (endpoint, mode) = (&args[1], args[2].as_str());
    let n: u64 = args.get(3).and_then(|a| a.parse().ok()).unwrap_or(10);
    match mode {
        "watch" | "ping" => {
            let s = connect(endpoint, None);
            let mut w = s.try_clone().unwrap();
            let (tx, rx) = mpsc::channel::<(String, u64)>();
            thread::spawn(move || {
                for l in BufReader::new(s).lines().map_while(Result::ok) {
                    let _ = tx.send((l, now_us()));
                }
            });
            let (mut push, mut rtt, mut gaps) = (vec![], vec![], vec![]);
            let (mut last_seq, mut missed, mut last_at) = (0, 0, None::<u64>);
            let (start, mut next_ping, mut sent) = (Instant::now(), Instant::now(), 0u64);
            let mut pings: Vec<(u64, u64)> = vec![];
            while start.elapsed() < Duration::from_secs(n) {
                if mode == "ping" && Instant::now() >= next_ping {
                    pings.push((sent, now_us()));
                    let _ = writeln!(w, "PING {sent}");
                    sent += 1;
                    next_ping += Duration::from_millis(50);
                }
                let Ok((line, at)) = rx.recv_timeout(Duration::from_millis(5)) else { continue };
                if let Some(id) = line.strip_prefix("PONG ").and_then(|r| r.split(' ').next()) {
                    let id: u64 = id.parse().unwrap();
                    if let Some((_, t0)) = pings.iter().find(|(i, _)| *i == id) {
                        rtt.push((at - t0) as f64 / 1000.0);
                    }
                } else if let (Some(seq), Some(us)) = (field(&line, "seq"), field(&line, "us")) {
                    push.push(at.saturating_sub(us) as f64 / 1000.0);
                    if last_seq != 0 && seq != last_seq + 1 { missed += seq - last_seq - 1; }
                    last_seq = seq;
                    if let Some(p) = last_at { gaps.push((at - p) as f64 / 1000.0); }
                    last_at = Some(at);
                }
            }
            println!("states received: {}, missed (seq gaps): {missed}", push.len());
            stats("push latency (extension tick -> client)", push);
            stats("interval between states", gaps);
            if mode == "ping" { stats("ping round trip (via main-thread tick)", rtt); }
        }
        "slow" => {
            let mut s = connect(endpoint, None);
            thread::sleep(Duration::from_secs(n));
            s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            let (mut bytes, mut buf) = (0usize, [0u8; 65536]);
            let closed = loop {
                match s.read(&mut buf) {
                    Ok(0) => break "EOF (extension closed the connection)",
                    Ok(k) => bytes += k,
                    Err(_) => break "still open (read timed out)",
                }
            };
            println!("never read for {n} s; then drained {bytes} bytes; connection: {closed}");
        }
        "churn" => {
            let start = Instant::now();
            for _ in 0..n {
                let s = connect(endpoint, None);
                let mut r = BufReader::new(s);
                let mut l = String::new();
                r.read_line(&mut l).unwrap();
                assert_eq!(l.trim(), "OK");
            }
            println!("{n} connect+auth+close cycles in {:.2} s", start.elapsed().as_secs_f64());
        }
        "badtoken" => {
            let s = connect(endpoint, Some("wrong"));
            let mut r = BufReader::new(s);
            let mut l = String::new();
            let got = r.read_line(&mut l).unwrap_or(0);
            println!("bad token: read {got} bytes (0 = closed without a reply, as required)");
        }
        _ => eprintln!("unknown mode"),
    }
}
