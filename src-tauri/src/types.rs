//! Shared serializable types for the Zavro Client frontend contract.
//! Field names/types mirror `src/types.ts` exactly.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A proxy node as shown in the UI. Extra connection details (uuid,
/// password, Reality keys, ...) live only in `raw_link` and are
/// re-parsed by `parser::parse_link` when a config is built.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub protocol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ping_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loss: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed_mbps: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub socks_port: u16,
    pub http_port: u16,
    /// "rule" | "global" | "direct" (lowercase, see frontend `routingModeLabel`)
    pub routing_mode: String,
    pub dns: Vec<String>,
    pub auto_update_hours: u32,
    pub system_proxy: bool,
    pub tun_mode: bool,
    pub log_level: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            socks_port: 10808,
            http_port: 10809,
            routing_mode: "rule".to_string(),
            dns: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
            auto_update_hours: 24,
            system_proxy: true,
            tun_mode: false,
            log_level: "warning".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub url: String,
    pub name: String,
    pub node_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub connected: bool,
    pub active_node_id: Option<String>,
    pub ping_ms: Option<f64>,
    pub dl_bps: f64,
    pub ul_bps: f64,
    pub routing_mode: String,
    pub system_proxy: bool,
    pub tun_mode: bool,
    pub core_running: bool,
    pub core_pid: Option<u32>,
    pub core_memory_mb: Option<f64>,
    pub core_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    pub node_id: String,
    pub ping_ms: Option<f64>,
    pub loss: f64, // percent 0.0 - 100.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// ISO-8601 UTC timestamp, e.g. "2026-11-25T14:52:00Z"
    pub ts: String,
    pub level: String,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficPoint {
    pub t: i64, // unix millis
    pub dl: f64,
    pub ul: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Traffic {
    pub dl_bps: f64,
    pub ul_bps: f64,
    pub points: Vec<TrafficPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreInfo {
    pub version: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub memory_mb: Option<f64>,
}

/// Routing rules as the frontend models them: a mode plus bypass lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRules {
    pub mode: String,
    #[serde(default)]
    pub bypass_domains: Vec<String>,
    #[serde(default)]
    pub bypass_ips: Vec<String>,
}

impl Default for RoutingRules {
    fn default() -> Self {
        Self {
            mode: "rule".to_string(),
            bypass_domains: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "10.0.0.0/8".to_string(),
                "192.168.0.0/16".to_string(),
            ],
            bypass_ips: vec!["geoip:private".to_string()],
        }
    }
}

/// Normalise a routing-mode string coming from the UI or stored data.
pub fn normalize_mode(mode: &str) -> String {
    match mode.to_ascii_lowercase().as_str() {
        "global" => "global".to_string(),
        "direct" => "direct".to_string(),
        "tun" => "tun".to_string(),
        _ => "rule".to_string(),
    }
}

/// Milliseconds since the Unix epoch.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Current UTC time as an ISO-8601 string (no chrono dependency).
pub fn iso_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    iso_from_unix(secs)
}

pub fn iso_from_unix(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // Civil-from-days algorithm (Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_097) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if mo <= 2 { y + 1 } else { y };
    format!("{year:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Best-effort unique id without an extra uuid dependency.
pub fn new_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{prefix}-{nanos:x}-{:x}", std::process::id())
}

pub fn log_entry(level: &str, msg: impl Into<String>) -> LogEntry {
    LogEntry {
        ts: iso_now(),
        level: level.to_string(),
        msg: msg.into(),
    }
}
