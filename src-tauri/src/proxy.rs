//! GNOME system proxy via `gsettings org.gnome.system.proxy`.
//! Real subprocess calls, verified by reading the values back.
//! If gsettings is missing (non-GNOME session) every call returns an
//! honest error instead of pretending success.

use std::process::Command;

const SCHEMA: &str = "org.gnome.system.proxy";
const SOCKS_SCHEMA: &str = "org.gnome.system.proxy.socks";
const HTTP_SCHEMA: &str = "org.gnome.system.proxy.http";
const HTTPS_SCHEMA: &str = "org.gnome.system.proxy.https";

fn gsettings(args: &[&str]) -> Result<String, String> {
    let out = Command::new("gsettings").args(args).output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "gsettings not found — this system does not use GNOME proxy settings".to_string()
        } else {
            format!("failed to run gsettings: {e}")
        }
    })?;
    if !out.status.success() {
        return Err(format!(
            "gsettings {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn set(schema: &str, key: &str, value: &str) -> Result<(), String> {
    gsettings(&["set", schema, key, value]).map(|_| ())
}

/// Apply the Zavro proxy: manual mode, SOCKS 127.0.0.1:10808,
/// HTTP/HTTPS 127.0.0.1:10809, localhost + private nets ignored.
pub fn apply_proxy(socks_port: u16, http_port: u16) -> Result<(), String> {
    set(SCHEMA, "mode", "'manual'")?;
    set(SOCKS_SCHEMA, "host", "'127.0.0.1'")?;
    set(SOCKS_SCHEMA, "port", &socks_port.to_string())?;
    set(HTTP_SCHEMA, "host", "'127.0.0.1'")?;
    set(HTTP_SCHEMA, "port", &http_port.to_string())?;
    set(HTTPS_SCHEMA, "host", "'127.0.0.1'")?;
    set(HTTPS_SCHEMA, "port", &http_port.to_string())?;
    set(
        SCHEMA,
        "ignore-hosts",
        "['localhost', '127.0.0.1', '10.0.0.0/8', '192.168.0.0/16']",
    )?;
    verify_proxy(socks_port, http_port)
}

/// Restore: proxy mode none.
pub fn restore_proxy() -> Result<(), String> {
    set(SCHEMA, "mode", "'none'")?;
    let mode = gsettings(&["get", SCHEMA, "mode"])?;
    if mode.trim_matches('\'') == "none" {
        Ok(())
    } else {
        Err(format!("proxy mode still {mode} after restore"))
    }
}

/// Read the current mode back ("none", "manual", ...).
pub fn current_mode() -> Result<String, String> {
    let mode = gsettings(&["get", SCHEMA, "mode"])?;
    Ok(mode.trim_matches('\'').to_string())
}

fn verify_proxy(socks_port: u16, http_port: u16) -> Result<(), String> {
    let mode = current_mode()?;
    if mode != "manual" {
        return Err(format!("proxy mode is {mode}, expected manual"));
    }
    let socks = gsettings(&["get", SOCKS_SCHEMA, "port"])?;
    if socks.trim() != socks_port.to_string() {
        return Err(format!(
            "socks port reads back as {socks}, expected {socks_port}"
        ));
    }
    let http = gsettings(&["get", HTTP_SCHEMA, "port"])?;
    if http.trim() != http_port.to_string() {
        return Err(format!(
            "http port reads back as {http}, expected {http_port}"
        ));
    }
    Ok(())
}
