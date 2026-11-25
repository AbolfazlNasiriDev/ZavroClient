//! Locate / probe the bundled Xray-core sidecar binary.
//! Lookup order: next to the app executable (Tauri `externalBin`
//! layout: `binaries/xray` beside the binary), then
//! `src-tauri/binaries/xray-x86_64-unknown-linux-gnu` (dev layout),
//! then `xray` on PATH. The version is never hardcoded — it is read
//! from `xray version` at runtime (upstream is XTLS/Xray-core,
//! latest stable v26.11.9 as of 2026-11-25; NEVER assume 1.8.8).

use std::path::PathBuf;
use std::process::Command;

pub const SIDECAR_TRIPLE: &str = "x86_64-unknown-linux-gnu";

/// Candidate locations for the xray binary, in priority order.
pub fn candidate_paths(exe_dir: Option<PathBuf>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(dir) = exe_dir {
        // Tauri externalBin: <exe_dir>/binaries/xray (and the
        // triple-suffixed dev name for good measure).
        out.push(dir.join("binaries").join("xray"));
        out.push(dir.join("binaries").join(format!("xray-{SIDECAR_TRIPLE}")));
    }
    if let Ok(cwd) = std::env::current_dir() {
        out.push(cwd.join("binaries").join(format!("xray-{SIDECAR_TRIPLE}")));
        out.push(cwd.join("binaries").join("xray"));
    }
    // Dev layout: <repo>/src-tauri/binaries/xray-<triple>
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        out.push(
            PathBuf::from(manifest)
                .join("binaries")
                .join(format!("xray-{SIDECAR_TRIPLE}")),
        );
    }
    out
}

/// Find the xray binary, or return an honest error string.
pub fn locate_xray(exe_dir: Option<PathBuf>) -> Result<PathBuf, String> {
    for p in candidate_paths(exe_dir) {
        if p.is_file() {
            return Ok(p);
        }
    }
    // Fall back to PATH.
    if let Ok(out) = Command::new("which").arg("xray").output() {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
    }
    if Command::new("xray").arg("version").output().is_ok() {
        return Ok(PathBuf::from("xray"));
    }
    Err("xray binary not found".to_string())
}

/// Run `xray version` and return its first line, e.g. "Xray 26.11.9 ...".
/// Returns None when the binary cannot be run.
pub fn xray_version(path: &std::path::Path) -> Option<String> {
    let out = Command::new(path).arg("version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines().next().map(|l| l.trim().to_string())
}

/// RSS of a process in MiB, read from /proc/<pid>/status (VmRSS).
pub fn process_memory_mb(pid: u32) -> Option<f64> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            let kb: f64 = rest.split_whitespace().next()?.parse().ok()?;
            return Some(kb / 1024.0);
        }
    }
    None
}

/// Validate a generated config with the real binary, if present:
/// `xray -test -config <path>`. `asset_dir` (geo data location) is
/// passed as XRAY_LOCATION_ASSET when given. Returns Ok(()) or the
/// binary's output.
pub fn test_config(
    xray: &std::path::Path,
    config_path: &std::path::Path,
    asset_dir: Option<&std::path::Path>,
) -> Result<(), String> {
    let mut cmd = Command::new(xray);
    cmd.arg("-test").arg("-config").arg(config_path);
    if let Some(dir) = asset_dir {
        cmd.env("XRAY_LOCATION_ASSET", dir);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("failed to run xray -test: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        Err(format!("xray -test failed: {err}{stdout}"))
    }
}
