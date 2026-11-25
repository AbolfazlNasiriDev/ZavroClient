//! Real telemetry: TCP connect probes and /proc/net/dev traffic.
//! No mock values — a failed probe counts as loss, and traffic is
//! measured from the kernel's interface counters.

use crate::types::TrafficPoint;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

/// One TCP probe: returns Some(connect time in ms) or None on failure.
fn probe_once(host: &str, port: u16, timeout: Duration) -> Option<f64> {
    let addrs: Vec<SocketAddr> = (host, port).to_socket_addrs().ok()?.collect();
    let addr = addrs.into_iter().next()?;
    let start = Instant::now();
    TcpStream::connect_timeout(&addr, timeout).ok()?;
    Some(start.elapsed().as_secs_f64() * 1000.0)
}

/// 3 probes. Returns (avg ping ms or None if all failed, loss percent).
pub fn ping_host(host: &str, port: u16) -> (Option<f64>, f64) {
    let mut ok = Vec::new();
    let mut failed = 0u32;
    for _ in 0..3 {
        match probe_once(host, port, Duration::from_secs(3)) {
            Some(ms) => ok.push(ms),
            None => failed += 1,
        }
    }
    let loss = failed as f64 / 3.0 * 100.0;
    let avg = if ok.is_empty() {
        None
    } else {
        Some(ok.iter().sum::<f64>() / ok.len() as f64)
    };
    (avg, loss)
}

/// Read total RX/TX bytes across all non-loopback interfaces.
pub fn read_net_dev() -> Option<(u64, u64)> {
    let text = std::fs::read_to_string("/proc/net/dev").ok()?;
    let mut rx = 0u64;
    let mut tx = 0u64;
    for line in text.lines().skip(2) {
        let mut parts = line.split_whitespace();
        let iface = parts.next()?.trim_end_matches(':');
        if iface == "lo" {
            continue;
        }
        let fields: Vec<&str> = parts.collect();
        if fields.len() < 9 {
            continue;
        }
        rx += fields[0].parse::<u64>().ok()?;
        tx += fields[8].parse::<u64>().ok()?;
    }
    Some((rx, tx))
}

/// Delta-based sample: (dl_bps, ul_bps) in bits per second, or None on
/// the first call (no baseline yet).
pub fn sample_traffic(prev: &mut Option<(u64, u64, Instant)>) -> (f64, f64, Option<TrafficPoint>) {
    let (rx, tx) = match read_net_dev() {
        Some(v) => v,
        None => return (0.0, 0.0, None),
    };
    let now = Instant::now();
    let now_ms = crate::types::now_ms();
    let result = match prev {
        Some((prx, ptx, t0)) => {
            let elapsed = now.duration_since(*t0).as_secs_f64();
            if elapsed <= 0.0 {
                (0.0, 0.0)
            } else {
                (
                    (rx.saturating_sub(*prx)) as f64 * 8.0 / elapsed,
                    (tx.saturating_sub(*ptx)) as f64 * 8.0 / elapsed,
                )
            }
        }
        None => (0.0, 0.0),
    };
    *prev = Some((rx, tx, now));
    let point = TrafficPoint {
        t: now_ms,
        dl: result.0,
        ul: result.1,
    };
    (result.0, result.1, Some(point))
}
