//! TUN mode via a pkexec helper script (`scripts/zavro-tun.sh`).
//! Rust never touches network interfaces directly: it invokes
//! `pkexec <script> up|down` and reports denial / missing pkexec
//! honestly. Teardown runs on disconnect and on app exit.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Locate the helper script: next to the exe (`scripts/zavro-tun.sh`),
/// then the dev layout under src-tauri/scripts/.
pub fn script_path(exe_dir: Option<&Path>) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(dir) = exe_dir {
        candidates.push(dir.join("scripts").join("zavro-tun.sh"));
        candidates.push(dir.join("resources").join("zavro-tun.sh"));
    }
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        candidates.push(PathBuf::from(manifest).join("scripts").join("zavro-tun.sh"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("scripts").join("zavro-tun.sh"));
    }
    candidates.into_iter().find(|p| p.is_file())
}

fn run_pkexec(script: &Path, action: &str) -> Result<(), String> {
    let out = Command::new("pkexec")
        .arg(script)
        .arg(action)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "pkexec not found — cannot create a TUN device without privilege escalation"
                    .to_string()
            } else {
                format!("failed to run pkexec: {e}")
            }
        })?;
    if out.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let code = out.status.code().map(|c| c.to_string()).unwrap_or_default();
        // pkexec exit 126/127 = dismissed / not authorized.
        if code == "126"
            || code == "127"
            || stderr.contains("dismissed")
            || stderr.contains("not authorized")
        {
            return Err(
                "TUN setup was dismissed or not authorized in the privilege prompt".to_string(),
            );
        }
        Err(format!(
            "zavro-tun.sh {action} failed (exit {code}): {stderr}"
        ))
    }
}

pub fn tun_up(exe_dir: Option<&Path>) -> Result<(), String> {
    let script = script_path(exe_dir).ok_or("zavro-tun.sh helper script not found")?;
    run_pkexec(&script, "up")
}

pub fn tun_down(exe_dir: Option<&Path>) -> Result<(), String> {
    let script = script_path(exe_dir).ok_or("zavro-tun.sh helper script not found")?;
    run_pkexec(&script, "down")
}
