//! Zavro Client backend: Tauri v2 commands, app state, lifecycle.
//! All commands return `Result<T, String>` with honest errors and
//! never panic on bad input.

mod config;
mod parser;
mod proxy;
mod store;
mod telemetry;
mod tun;
mod types;
mod xray;

use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{Manager, State};
use types::*;

pub struct AppState {
    pub nodes: Mutex<Vec<Node>>,
    pub subscriptions: Mutex<Vec<Subscription>>,
    pub settings: Mutex<Settings>,
    pub routing: Mutex<RoutingRules>,
    pub logs: Arc<Mutex<VecDeque<LogEntry>>>,
    pub traffic_points: Mutex<Vec<TrafficPoint>>,
    pub traffic_last: Mutex<Option<(u64, u64, Instant)>>,
    pub xray_child: Mutex<Option<Child>>,
    pub core_version: Mutex<Option<String>>,
    pub active_node_id: Mutex<Option<String>>,
    pub data_dir: PathBuf,
    pub exe_dir: Option<PathBuf>,
    /// Directory holding geoip.dat / geosite.dat (Tauri resource dir),
    /// passed to xray as XRAY_LOCATION_ASSET. None if not found.
    pub asset_dir: Option<PathBuf>,
}

impl AppState {
    fn new(
        data_dir: PathBuf,
        exe_dir: Option<PathBuf>,
        asset_dir: Option<PathBuf>,
        data: store::StoreData,
    ) -> Self {
        let core_version = xray::locate_xray(exe_dir.clone())
            .ok()
            .and_then(|p| xray::xray_version(&p));
        let mut logs = VecDeque::new();
        logs.push_back(log_entry(
            "info",
            format!(
                "Zavro Client started; xray core: {}",
                core_version
                    .clone()
                    .unwrap_or_else(|| "not found".to_string())
            ),
        ));
        Self {
            nodes: Mutex::new(data.nodes),
            subscriptions: Mutex::new(data.subscriptions),
            settings: Mutex::new(data.settings),
            routing: Mutex::new(data.routing),
            logs: Arc::new(Mutex::new(logs)),
            traffic_points: Mutex::new(Vec::new()),
            traffic_last: Mutex::new(None),
            xray_child: Mutex::new(None),
            core_version: Mutex::new(core_version),
            active_node_id: Mutex::new(None),
            data_dir,
            exe_dir,
            asset_dir,
        }
    }

    fn log(&self, level: &str, msg: impl Into<String>) {
        let entry = log_entry(level, msg);
        let mut logs = self.logs.lock().unwrap_or_else(|e| e.into_inner());
        logs.push_back(entry);
        while logs.len() > 500 {
            logs.pop_front();
        }
    }

    fn persist(&self) -> Result<(), String> {
        let data = store::StoreData {
            nodes: self.nodes.lock().unwrap_or_else(|e| e.into_inner()).clone(),
            subscriptions: self
                .subscriptions
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone(),
            settings: self
                .settings
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone(),
            routing: self
                .routing
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone(),
        };
        store::save(&self.data_dir, &data)
    }

    /// (running, pid) — reaps a child that already exited.
    fn core_state(&self) -> (bool, Option<u32>) {
        let mut guard = self.xray_child.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(child) = guard.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => {
                    *guard = None;
                    (false, None)
                }
                Ok(None) => (true, Some(child.id())),
                Err(_) => (false, None),
            }
        } else {
            (false, None)
        }
    }

    fn kill_core(&self) {
        let mut guard = self.xray_child.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Stop the core, restore the system proxy and tear the TUN device
/// down. Safe to call repeatedly (disconnect / app exit).
fn cleanup(state: &AppState) {
    state.kill_core();
    {
        let mut active = state
            .active_node_id
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *active = None;
    }
    if let Err(e) = proxy::restore_proxy() {
        state.log("warn", format!("proxy restore on exit: {e}"));
    }
    let tun_on = state
        .settings
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .tun_mode;
    if tun_on {
        if let Err(e) = tun::tun_down(state.exe_dir.as_deref()) {
            state.log("warn", format!("tun teardown on exit: {e}"));
        }
    }
    state.log("info", "Disconnected: core stopped, system proxy restored");
}

// ---------------------------------------------------------------------------
// Status / nodes
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_status(state: State<'_, AppState>) -> Result<Status, String> {
    let (core_running, core_pid) = state.core_state();
    let active_node_id = state
        .active_node_id
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let settings = state
        .settings
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ping_ms = active_node_id.as_ref().and_then(|id| {
        state
            .nodes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .find(|n| &n.id == id)
            .and_then(|n| n.ping_ms)
    });
    let (dl_bps, ul_bps) = state
        .traffic_points
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .last()
        .map(|p| (p.dl, p.ul))
        .unwrap_or((0.0, 0.0));
    let core_version = state
        .core_version
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_default();
    let core_memory_mb = core_pid.and_then(xray::process_memory_mb);
    Ok(Status {
        connected: core_running && active_node_id.is_some(),
        active_node_id,
        ping_ms,
        dl_bps,
        ul_bps,
        routing_mode: settings.routing_mode,
        system_proxy: settings.system_proxy,
        tun_mode: settings.tun_mode,
        core_running,
        core_pid,
        core_memory_mb,
        core_version,
    })
}

#[tauri::command]
fn list_nodes(state: State<'_, AppState>) -> Result<Vec<Node>, String> {
    Ok(state
        .nodes
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone())
}

fn validate_node(node: &Node) -> Result<(), String> {
    if node.name.trim().is_empty() {
        return Err("node name is empty".to_string());
    }
    if node.address.trim().is_empty() {
        return Err("node address is empty".to_string());
    }
    if node.port == 0 {
        return Err("node port is 0".to_string());
    }
    match node.protocol.as_str() {
        "vless" | "vmess" | "trojan" | "shadowsocks" => Ok(()),
        other => Err(format!("unsupported protocol: {other}")),
    }
}

#[tauri::command]
fn add_node(state: State<'_, AppState>, node: Node) -> Result<Node, String> {
    validate_node(&node)?;
    if let Some(link) = node.raw_link.as_deref() {
        if !link.trim().is_empty() {
            parser::parse_link(link, node.group.as_deref().unwrap_or("Zavro"))?;
        }
    }
    let mut node = node;
    if node.id.trim().is_empty() {
        node.id = new_id("node");
    }
    if node.status.is_none() {
        node.status = Some("idle".to_string());
    }
    {
        let mut nodes = state.nodes.lock().unwrap_or_else(|e| e.into_inner());
        if nodes.iter().any(|n| n.id == node.id) {
            return Err(format!("a node with id {} already exists", node.id));
        }
        nodes.push(node.clone());
    }
    state.persist()?;
    state.log("info", format!("Node added: {}", node.name));
    Ok(node)
}

#[tauri::command]
fn update_node(state: State<'_, AppState>, node: Node) -> Result<Node, String> {
    validate_node(&node)?;
    if let Some(link) = node.raw_link.as_deref() {
        if !link.trim().is_empty() {
            parser::parse_link(link, node.group.as_deref().unwrap_or("Zavro"))?;
        }
    }
    {
        let mut nodes = state.nodes.lock().unwrap_or_else(|e| e.into_inner());
        let pos = nodes
            .iter()
            .position(|n| n.id == node.id)
            .ok_or_else(|| format!("no node with id {}", node.id))?;
        nodes[pos] = node.clone();
    }
    state.persist()?;
    Ok(node)
}

#[tauri::command]
fn delete_node(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let was_active = state
        .active_node_id
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_deref()
        == Some(id.as_str());
    if was_active {
        disconnect_inner(&state)?;
    }
    {
        let mut nodes = state.nodes.lock().unwrap_or_else(|e| e.into_inner());
        let before = nodes.len();
        nodes.retain(|n| n.id != id);
        if nodes.len() == before {
            return Err(format!("no node with id {id}"));
        }
    }
    state.persist()?;
    Ok(())
}

#[tauri::command]
fn import_link(state: State<'_, AppState>, link: String) -> Result<Node, String> {
    let parsed = parser::parse_link(&link, "Zavro")?;
    let node = parsed.to_node();
    {
        let mut nodes = state.nodes.lock().unwrap_or_else(|e| e.into_inner());
        if nodes
            .iter()
            .any(|n| n.raw_link.as_deref() == Some(link.trim()))
        {
            return Err("this link is already imported".to_string());
        }
        nodes.push(node.clone());
    }
    state.persist()?;
    state.log("info", format!("Link imported: {}", node.name));
    Ok(node)
}

// ---------------------------------------------------------------------------
// Subscriptions (fetched with curl — no TLS stack in std)
// ---------------------------------------------------------------------------

fn fetch_url(url: &str) -> Result<String, String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("subscription URL must start with http:// or https://".to_string());
    }
    let out = Command::new("curl")
        .args(["-sL", "--max-time", "20", url])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "curl not found — cannot fetch subscriptions".to_string()
            } else {
                format!("failed to run curl: {e}")
            }
        })?;
    if !out.status.success() {
        return Err(format!(
            "subscription fetch failed (curl exit {}): {}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let body = String::from_utf8_lossy(&out.stdout).into_owned();
    if body.trim().is_empty() {
        return Err("subscription returned an empty body".to_string());
    }
    Ok(body)
}

fn subscription_name(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| "Subscription".to_string())
}

fn import_nodes_from_body(
    state: &State<'_, AppState>,
    body: &str,
    group: &str,
) -> Result<Vec<Node>, String> {
    let parsed = parser::decode_subscription_blob(body, group)?;
    let mut added = Vec::new();
    {
        let mut nodes = state.nodes.lock().unwrap_or_else(|e| e.into_inner());
        for p in parsed {
            let node = p.to_node();
            let duplicate = nodes.iter().any(|n| {
                n.raw_link.as_deref() == node.raw_link.as_deref()
                    || (n.address == node.address && n.port == node.port)
            });
            if !duplicate {
                nodes.push(node.clone());
                added.push(node);
            }
        }
    }
    state.persist()?;
    Ok(added)
}

#[tauri::command]
fn import_subscription(state: State<'_, AppState>, url: String) -> Result<Vec<Node>, String> {
    let body = fetch_url(&url)?;
    let added = import_nodes_from_body(&state, &body, &subscription_name(&url))?;
    state.log(
        "info",
        format!(
            "Subscription imported from {url}: {} new node(s)",
            added.len()
        ),
    );
    Ok(added)
}

#[tauri::command]
fn update_subscriptions(state: State<'_, AppState>) -> Result<Vec<Node>, String> {
    let subs: Vec<Subscription> = state
        .subscriptions
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    if subs.is_empty() {
        return Ok(Vec::new());
    }
    let mut all_added = Vec::new();
    let mut failures = Vec::new();
    for sub in &subs {
        match fetch_url(&sub.url) {
            Ok(body) => match import_nodes_from_body(&state, &body, &sub.name) {
                Ok(added) => {
                    let count = added.len();
                    all_added.extend(added);
                    let mut subs_guard = state
                        .subscriptions
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    if let Some(s) = subs_guard.iter_mut().find(|s| s.id == sub.id) {
                        s.node_count += count;
                        s.last_updated = Some(iso_now());
                    }
                }
                Err(e) => failures.push(format!("{}: {e}", sub.url)),
            },
            Err(e) => failures.push(format!("{}: {e}", sub.url)),
        }
    }
    state.persist()?;
    if all_added.is_empty() && !failures.is_empty() {
        return Err(format!(
            "all subscription updates failed: {}",
            failures.join("; ")
        ));
    }
    if !failures.is_empty() {
        state.log(
            "warn",
            format!("Some subscriptions failed: {}", failures.join("; ")),
        );
    }
    Ok(all_added)
}

#[tauri::command]
fn get_subscriptions(state: State<'_, AppState>) -> Result<Vec<Subscription>, String> {
    Ok(state
        .subscriptions
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone())
}

#[tauri::command]
fn add_subscription(state: State<'_, AppState>, url: String) -> Result<Subscription, String> {
    let body = fetch_url(&url)?;
    let name = subscription_name(&url);
    let added = import_nodes_from_body(&state, &body, &name)?;
    let sub = Subscription {
        id: new_id("sub"),
        url: url.clone(),
        name,
        node_count: added.len(),
        last_updated: Some(iso_now()),
    };
    {
        let mut subs = state
            .subscriptions
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        subs.push(sub.clone());
    }
    state.persist()?;
    state.log("info", format!("Subscription added: {}", sub.url));
    Ok(sub)
}

#[tauri::command]
fn delete_subscription(state: State<'_, AppState>, id: String) -> Result<(), String> {
    {
        let mut subs = state
            .subscriptions
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let before = subs.len();
        subs.retain(|s| s.id != id);
        if subs.len() == before {
            return Err(format!("no subscription with id {id}"));
        }
    }
    state.persist()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Ping
// ---------------------------------------------------------------------------

fn ping_and_store(state: &State<'_, AppState>, id: &str) -> Result<PingResult, String> {
    let node = state
        .nodes
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|n| n.id == id)
        .cloned()
        .ok_or_else(|| format!("no node with id {id}"))?;
    let (ping_ms, loss) = telemetry::ping_host(&node.address, node.port);
    {
        let mut nodes = state.nodes.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(n) = nodes.iter_mut().find(|n| n.id == id) {
            n.ping_ms = ping_ms;
            n.loss = Some(loss);
        }
    }
    state.persist()?;
    Ok(PingResult {
        node_id: id.to_string(),
        ping_ms,
        loss,
    })
}

#[tauri::command]
fn check_ping(state: State<'_, AppState>, id: String) -> Result<PingResult, String> {
    ping_and_store(&state, &id)
}

#[tauri::command]
fn check_ping_all(state: State<'_, AppState>) -> Result<Vec<PingResult>, String> {
    let nodes: Vec<Node> = state
        .nodes
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    // One thread per node; std threads only, no async runtime.
    let results: Vec<PingResult> = std::thread::scope(|scope| {
        let handles: Vec<_> = nodes
            .iter()
            .map(|n| {
                let (addr, port, id) = (n.address.clone(), n.port, n.id.clone());
                scope.spawn(move || {
                    let (ping_ms, loss) = telemetry::ping_host(&addr, port);
                    PingResult {
                        node_id: id,
                        ping_ms,
                        loss,
                    }
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| {
                h.join().unwrap_or(PingResult {
                    node_id: String::new(),
                    ping_ms: None,
                    loss: 100.0,
                })
            })
            .collect()
    });
    {
        let mut nodes_guard = state.nodes.lock().unwrap_or_else(|e| e.into_inner());
        for r in &results {
            if let Some(n) = nodes_guard.iter_mut().find(|n| n.id == r.node_id) {
                n.ping_ms = r.ping_ms;
                n.loss = Some(r.loss);
            }
        }
    }
    state.persist()?;
    Ok(results)
}

// ---------------------------------------------------------------------------
// Connect / disconnect / proxy / TUN / routing
// ---------------------------------------------------------------------------

fn disconnect_inner(state: &State<'_, AppState>) -> Result<(), String> {
    cleanup(state);
    state.persist()?;
    Ok(())
}

#[tauri::command]
fn connect(state: State<'_, AppState>, node_id: String) -> Result<(), String> {
    // If already connected, stop the old core first.
    if state.core_state().0 {
        cleanup(&state);
    }
    let node = state
        .nodes
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|n| n.id == node_id)
        .cloned()
        .ok_or_else(|| format!("no node with id {node_id}"))?;
    let link = node
        .raw_link
        .clone()
        .filter(|l| !l.trim().is_empty())
        .ok_or_else(|| format!("node {} has no link to connect with", node.name))?;
    let parsed = parser::parse_link(&link, node.group.as_deref().unwrap_or("Zavro"))?;
    let settings = state
        .settings
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let routing = state
        .routing
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();

    let xray_path = xray::locate_xray(state.exe_dir.clone())?;
    let config_text = config::build_config_string(&parsed, &settings, &routing, &state.data_dir)?;
    let config_path = state.data_dir.join("config.json");
    std::fs::create_dir_all(&state.data_dir)
        .map_err(|e| format!("cannot create app data dir: {e}"))?;
    std::fs::write(&config_path, &config_text)
        .map_err(|e| format!("cannot write xray config: {e}"))?;
    xray::test_config(&xray_path, &config_path, state.asset_dir.as_deref())?;

    let mut cmd = Command::new(&xray_path);
    cmd.arg("-config").arg(&config_path);
    if let Some(dir) = &state.asset_dir {
        cmd.env("XRAY_LOCATION_ASSET", dir);
    }
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to start xray: {e}"))?;

    // Stream xray stdout/stderr into the 500-entry ring buffer.
    let logs = Arc::clone(&state.logs);
    if let Some(stdout) = child.stdout.take() {
        let logs = Arc::clone(&logs);
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                push_log(&logs, "info", line);
            }
        });
    }
    if let Some(stderr) = child.stderr.take() {
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                push_log(&logs, "error", line);
            }
        });
    }
    {
        let mut guard = state.xray_child.lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(child);
    }
    {
        let mut active = state
            .active_node_id
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *active = Some(node.id.clone());
    }
    {
        let mut nodes = state.nodes.lock().unwrap_or_else(|e| e.into_inner());
        for n in nodes.iter_mut() {
            n.status = Some(if n.id == node.id {
                "connected".to_string()
            } else {
                "idle".to_string()
            });
        }
    }
    state.log("info", format!("Connected to {} via xray", node.name));

    // System proxy (rollback the core if it cannot be applied).
    if settings.system_proxy {
        if let Err(e) = proxy::apply_proxy(settings.socks_port, settings.http_port) {
            cleanup(&state);
            return Err(format!("connected to core but system proxy failed: {e}"));
        }
    }
    // TUN (rollback the core if privilege escalation fails).
    if settings.tun_mode {
        if let Err(e) = tun::tun_up(state.exe_dir.as_deref()) {
            cleanup(&state);
            return Err(format!("connected to core but TUN setup failed: {e}"));
        }
    }
    state.persist()?;
    Ok(())
}

fn push_log(logs: &Arc<Mutex<VecDeque<LogEntry>>>, level: &str, msg: String) {
    let mut guard = logs.lock().unwrap_or_else(|e| e.into_inner());
    guard.push_back(log_entry(level, msg));
    while guard.len() > 500 {
        guard.pop_front();
    }
}

#[tauri::command]
fn disconnect(state: State<'_, AppState>) -> Result<(), String> {
    disconnect_inner(&state)
}

#[tauri::command]
fn set_system_proxy(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    let settings = state
        .settings
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    if enabled {
        proxy::apply_proxy(settings.socks_port, settings.http_port)?;
    } else {
        proxy::restore_proxy()?;
    }
    {
        let mut s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        s.system_proxy = enabled;
    }
    state.persist()?;
    state.log(
        "info",
        format!(
            "System proxy {}",
            if enabled { "enabled" } else { "disabled" }
        ),
    );
    Ok(())
}

#[tauri::command]
fn set_tun(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    if enabled {
        tun::tun_up(state.exe_dir.as_deref())?;
    } else {
        tun::tun_down(state.exe_dir.as_deref())?;
    }
    {
        let mut s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        s.tun_mode = enabled;
    }
    state.persist()?;
    state.log(
        "info",
        format!(
            "TUN mode {} (takes effect for the Xray inbound on next connect)",
            if enabled { "enabled" } else { "disabled" }
        ),
    );
    Ok(())
}

#[tauri::command]
fn set_routing_mode(state: State<'_, AppState>, mode: String) -> Result<(), String> {
    let mode = normalize_mode(&mode);
    {
        let mut s = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        s.routing_mode = mode.clone();
        let mut r = state.routing.lock().unwrap_or_else(|e| e.into_inner());
        r.mode = mode.clone();
    }
    state.persist()?;
    state.log("info", format!("Routing mode set to {mode}"));
    Ok(())
}

// ---------------------------------------------------------------------------
// Logs / traffic / settings / core info / routing rules
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_logs(state: State<'_, AppState>) -> Result<Vec<LogEntry>, String> {
    Ok(state
        .logs
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .cloned()
        .collect())
}

#[tauri::command]
fn clear_logs(state: State<'_, AppState>) -> Result<(), String> {
    state.logs.lock().unwrap_or_else(|e| e.into_inner()).clear();
    Ok(())
}

#[tauri::command]
fn get_traffic(state: State<'_, AppState>) -> Result<Traffic, String> {
    let (dl_bps, ul_bps, point) = {
        let mut last = state.traffic_last.lock().unwrap_or_else(|e| e.into_inner());
        telemetry::sample_traffic(&mut last)
    };
    if let Some(point) = point {
        let mut points = state
            .traffic_points
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        points.push(point);
        while points.len() > 60 {
            points.remove(0);
        }
        let points = points.clone();
        return Ok(Traffic {
            dl_bps,
            ul_bps,
            points,
        });
    }
    Ok(Traffic {
        dl_bps,
        ul_bps,
        points: state
            .traffic_points
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone(),
    })
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    Ok(state
        .settings
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone())
}

#[tauri::command]
fn save_settings(state: State<'_, AppState>, s: Settings) -> Result<(), String> {
    if s.socks_port == 0 || s.http_port == 0 {
        return Err("proxy ports must be non-zero".to_string());
    }
    if s.socks_port == s.http_port {
        return Err("SOCKS and HTTP ports must differ".to_string());
    }
    let mut s = s;
    s.routing_mode = normalize_mode(&s.routing_mode);
    if s.dns.is_empty() {
        s.dns = vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()];
    }
    {
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        *settings = s.clone();
        let mut routing = state.routing.lock().unwrap_or_else(|e| e.into_inner());
        routing.mode = s.routing_mode.clone();
    }
    state.persist()?;
    Ok(())
}

#[tauri::command]
fn get_core_info(state: State<'_, AppState>) -> Result<CoreInfo, String> {
    let (running, pid) = state.core_state();
    let version = state
        .core_version
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_default();
    let memory_mb = pid.and_then(xray::process_memory_mb);
    Ok(CoreInfo {
        version,
        running,
        pid,
        memory_mb,
    })
}

#[tauri::command]
fn get_routing_rules(state: State<'_, AppState>) -> Result<RoutingRules, String> {
    Ok(state
        .routing
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone())
}

#[tauri::command]
fn save_routing_rules(state: State<'_, AppState>, r: RoutingRules) -> Result<(), String> {
    let r = RoutingRules {
        mode: normalize_mode(&r.mode),
        bypass_domains: r.bypass_domains,
        bypass_ips: r.bypass_ips,
    };
    {
        // Lock order is always settings -> routing (see set_routing_mode).
        let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        settings.routing_mode = r.mode.clone();
        let mut routing = state.routing.lock().unwrap_or_else(|e| e.into_inner());
        *routing = r.clone();
    }
    state.persist()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// App entry
// ---------------------------------------------------------------------------

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_status,
            list_nodes,
            add_node,
            update_node,
            delete_node,
            import_link,
            import_subscription,
            update_subscriptions,
            get_subscriptions,
            add_subscription,
            delete_subscription,
            check_ping_all,
            check_ping,
            connect,
            disconnect,
            set_system_proxy,
            set_tun,
            set_routing_mode,
            get_logs,
            clear_logs,
            get_traffic,
            get_settings,
            save_settings,
            get_core_info,
            get_routing_rules,
            save_routing_rules
        ])
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            let _ = std::fs::create_dir_all(&data_dir);
            let data = store::load(&data_dir);
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()));
            // Geo data location for Xray (geosite:/geoip: routing rules
            // need geoip.dat + geosite.dat): Tauri resource dir first,
            // then <exe>/resources, then the dev layout.
            let asset_dir = {
                let mut candidates: Vec<PathBuf> = Vec::new();
                if let Ok(dir) = app.path().resource_dir() {
                    candidates.push(dir);
                }
                if let Some(dir) = &exe_dir {
                    candidates.push(dir.join("resources"));
                    if let Some(parent) = dir.parent() {
                        candidates.push(parent.join("resources"));
                    }
                }
                if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
                    candidates.push(PathBuf::from(manifest).join("resources"));
                }
                candidates
                    .into_iter()
                    .find(|d| d.join("geosite.dat").is_file())
            };
            // Persist the seed on first run so a later launch reloads it.
            if !store::store_path(&data_dir).exists() {
                let _ = store::save(&data_dir, &data);
            }
            app.manage(AppState::new(data_dir, exe_dir, asset_dir, data));
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            use tauri::RunEvent;
            match event {
                RunEvent::ExitRequested { .. } | RunEvent::Exit => {
                    let state = app_handle.state::<AppState>();
                    cleanup(&state);
                    let _ = state.persist();
                }
                _ => {}
            }
        });
}
