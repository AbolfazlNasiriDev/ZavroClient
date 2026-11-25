//! Build a valid Xray `config.json` from a selected node + settings.
//! Schema follows Xray-core (XTLS/Xray-core, v26.x): log / dns /
//! inbounds / outbounds / routing. No fake values: every field comes
//! from the node, the settings, or the routing rules.

use crate::parser::ParsedNode;
use crate::types::{RoutingRules, Settings};
use serde_json::{json, Value};
use std::path::Path;

/// Build the Xray config as a JSON value.
pub fn build_config(
    node: &ParsedNode,
    settings: &Settings,
    routing: &RoutingRules,
    log_dir: &Path,
) -> Value {
    let mut inbounds = vec![
        json!({
            "tag": "socks-in",
            "port": settings.socks_port,
            "listen": "127.0.0.1",
            "protocol": "socks",
            "settings": { "auth": "noauth", "udp": true, "ip": "127.0.0.1" },
            "sniffing": { "enabled": true, "destOverride": ["http", "tls", "quic"] }
        }),
        json!({
            "tag": "http-in",
            "port": settings.http_port,
            "listen": "127.0.0.1",
            "protocol": "http",
            "settings": {},
            "sniffing": { "enabled": true, "destOverride": ["http", "tls", "quic"] }
        }),
    ];
    if settings.tun_mode {
        inbounds.push(json!({
            "tag": "tun-in",
            "protocol": "tun",
            "settings": { "address": "10.0.0.1/30", "mtu": 1500 },
            "sniffing": { "enabled": true, "destOverride": ["http", "tls", "quic"] }
        }));
    }

    let mut rules: Vec<Value> = vec![
        json!({
            "type": "field",
            "domain": ["geosite:category-ads-all"],
            "outboundTag": "block"
        }),
        json!({
            "type": "field",
            "ip": ["geoip:private"],
            "outboundTag": "direct"
        }),
    ];
    if !routing.bypass_domains.is_empty() {
        rules.push(json!({
            "type": "field",
            "domain": routing.bypass_domains,
            "outboundTag": "direct"
        }));
    }
    let bypass_ips: Vec<String> = routing
        .bypass_ips
        .iter()
        .filter(|ip| ip.as_str() != "geoip:private")
        .cloned()
        .collect();
    if !bypass_ips.is_empty() {
        rules.push(json!({
            "type": "field",
            "ip": bypass_ips,
            "outboundTag": "direct"
        }));
    }

    let final_tag = match routing.mode.as_str() {
        "global" => "proxy",
        "direct" => "direct",
        _ => "proxy", // "rule": unmatched traffic goes through the proxy
    };

    let dns_servers: Vec<Value> = if settings.dns.is_empty() {
        vec![json!("1.1.1.1"), json!("8.8.8.8")]
    } else {
        settings.dns.iter().map(|d| json!(d)).collect()
    };

    json!({
        "log": {
            "loglevel": settings.log_level,
            "access": log_dir.join("xray-access.log").to_string_lossy(),
            "error": log_dir.join("xray-error.log").to_string_lossy()
        },
        "dns": { "servers": dns_servers, "queryStrategy": "UseIPv4" },
        "inbounds": inbounds,
        "outbounds": [
            build_proxy_outbound(node),
            { "tag": "direct", "protocol": "freedom", "settings": {} },
            { "tag": "block", "protocol": "blackhole", "settings": { "response": { "type": "http" } } }
        ],
        "routing": {
            "domainStrategy": "IPIfNonMatch",
            "rules": rules,
            "final": final_tag
        }
    })
}

/// Serialize the config to pretty JSON text.
pub fn build_config_string(
    node: &ParsedNode,
    settings: &Settings,
    routing: &RoutingRules,
    log_dir: &Path,
) -> Result<String, String> {
    let value = build_config(node, settings, routing, log_dir);
    serde_json::to_string_pretty(&value).map_err(|e| format!("config serialize failed: {e}"))
}

fn stream_settings(node: &ParsedNode) -> Value {
    let mut ss = json!({ "network": node.transport });
    match node.security.as_str() {
        "reality" => {
            ss["security"] = json!("reality");
            let mut reality = json!({});
            if !node.sni.is_empty() {
                reality["serverName"] = json!(node.sni);
            }
            if !node.fingerprint.is_empty() {
                reality["fingerprint"] = json!(node.fingerprint);
            }
            if !node.public_key.is_empty() {
                reality["publicKey"] = json!(node.public_key);
            }
            if !node.short_id.is_empty() {
                reality["shortId"] = json!(node.short_id);
            }
            ss["realitySettings"] = reality;
        }
        "tls" => {
            ss["security"] = json!("tls");
            let mut tls = json!({ "allowInsecure": false });
            if !node.sni.is_empty() {
                tls["serverName"] = json!(node.sni);
            }
            if !node.fingerprint.is_empty() {
                tls["fingerprint"] = json!(node.fingerprint);
            }
            if !node.alpn.is_empty() {
                tls["alpn"] = json!(vec![node.alpn.clone()]);
            }
            ss["tlsSettings"] = tls;
        }
        _ => {
            ss["security"] = json!("none");
        }
    }
    match node.transport.as_str() {
        "ws" => {
            let mut ws = json!({});
            if !node.path.is_empty() {
                ws["path"] = json!(node.path);
            }
            if !node.host.is_empty() {
                ws["headers"] = json!({ "Host": node.host });
            }
            ss["wsSettings"] = ws;
        }
        "grpc" => {
            let mut grpc = json!({ "multiMode": false });
            if !node.service_name.is_empty() {
                grpc["serviceName"] = json!(node.service_name);
            }
            ss["grpcSettings"] = grpc;
        }
        // "tcp" needs no extra stream settings; the xtls-rprx-vision
        // flow (if any) lives on the VLESS user object, not here.
        _ => {}
    }
    ss
}

fn build_proxy_outbound(node: &ParsedNode) -> Value {
    match node.protocol.as_str() {
        "vless" => {
            let mut user = json!({ "id": node.uuid, "encryption": "none", "level": 0 });
            if node.flow == "xtls-rprx-vision" {
                user["flow"] = json!("xtls-rprx-vision");
            }
            json!({
                "tag": "proxy",
                "protocol": "vless",
                "settings": { "vnext": [{ "address": node.address, "port": node.port, "users": [user] }] },
                "streamSettings": stream_settings(node)
            })
        }
        "vmess" => {
            json!({
                "tag": "proxy",
                "protocol": "vmess",
                "settings": { "vnext": [{ "address": node.address, "port": node.port, "users": [{ "id": node.uuid, "alterId": node.alter_id, "security": "auto", "level": 0 }] }] },
                "streamSettings": stream_settings(node)
            })
        }
        "trojan" => {
            json!({
                "tag": "proxy",
                "protocol": "trojan",
                "settings": { "servers": [{ "address": node.address, "port": node.port, "password": node.password, "level": 0 }] },
                "streamSettings": stream_settings(node)
            })
        }
        "shadowsocks" => {
            json!({
                "tag": "proxy",
                "protocol": "shadowsocks",
                "settings": { "servers": [{ "address": node.address, "port": node.port, "method": node.method, "password": node.password, "level": 0 }] },
                "streamSettings": stream_settings(node)
            })
        }
        other => {
            // Unknown protocol: never invent an outbound. Caller must
            // have validated the protocol via parse_link first.
            json!({
                "tag": "proxy",
                "protocol": other,
                "settings": {}
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_link;
    use std::path::PathBuf;

    fn settings() -> Settings {
        Settings::default()
    }

    #[test]
    fn vless_reality_config_is_valid_json_schema() {
        let link = "vless://550e8400-e29b-41d4-a716-446655440000@de-01.zavro.net:443?encryption=none&security=reality&sni=example.com&fp=chrome&pbk=bS9sDoQxkOfKpGVR_9ou_IH1IMj9iz-5E54b_JweDCA&sid=1234abcd&flow=xtls-rprx-vision&type=tcp#DE-01";
        let node = parse_link(link, "Zavro").unwrap();
        let cfg = build_config(
            &node,
            &settings(),
            &RoutingRules::default(),
            &PathBuf::from("/tmp"),
        );
        assert_eq!(cfg["outbounds"][0]["protocol"], "vless");
        assert_eq!(cfg["outbounds"][0]["streamSettings"]["security"], "reality");
        assert_eq!(
            cfg["outbounds"][0]["streamSettings"]["realitySettings"]["publicKey"],
            "bS9sDoQxkOfKpGVR_9ou_IH1IMj9iz-5E54b_JweDCA"
        );
        assert_eq!(cfg["inbounds"][0]["port"], 10808);
        assert_eq!(cfg["inbounds"][1]["port"], 10809);
        assert_eq!(cfg["routing"]["final"], "proxy");
        assert_eq!(cfg["dns"]["servers"][0], "1.1.1.1");
        // Round-trips as JSON text.
        let text = build_config_string(
            &node,
            &settings(),
            &RoutingRules::default(),
            &PathBuf::from("/tmp"),
        )
        .unwrap();
        let reparsed: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(reparsed, cfg);
    }

    #[test]
    fn tun_inbound_only_when_enabled() {
        let link = "trojan://pw@nl-02.zavro.net:443#NL-02";
        let node = parse_link(link, "Zavro").unwrap();
        let mut s = settings();
        let cfg = build_config(&node, &s, &RoutingRules::default(), &PathBuf::from("/tmp"));
        assert_eq!(cfg["inbounds"].as_array().unwrap().len(), 2);
        s.tun_mode = true;
        let cfg = build_config(&node, &s, &RoutingRules::default(), &PathBuf::from("/tmp"));
        assert_eq!(cfg["inbounds"].as_array().unwrap().len(), 3);
        assert_eq!(cfg["inbounds"][2]["protocol"], "tun");
        assert_eq!(cfg["inbounds"][2]["settings"]["address"], "10.0.0.1/30");
    }

    #[test]
    fn direct_mode_final_is_direct() {
        let link = "trojan://pw@nl-02.zavro.net:443#NL-02";
        let node = parse_link(link, "Zavro").unwrap();
        let mut r = RoutingRules::default();
        r.mode = "direct".to_string();
        let cfg = build_config(&node, &settings(), &r, &PathBuf::from("/tmp"));
        assert_eq!(cfg["routing"]["final"], "direct");
    }

    /// Validate a generated config with the REAL xray binary, when the
    /// test env provides one: `ZAVRO_XRAY_BIN=/path/to/xray cargo test`.
    /// Skipped silently when unset (CI stages the binary before testing).
    #[test]
    fn generated_config_passes_real_xray_test() {
        let xray_bin = match std::env::var("ZAVRO_XRAY_BIN") {
            Ok(p) if !p.is_empty() => PathBuf::from(p),
            _ => return,
        };
        let asset_dir = xray_bin
            .parent()
            .map(|d| d.to_path_buf())
            .filter(|d| d.join("geosite.dat").is_file())
            .or_else(|| {
                // Dev layout: geo files live in src-tauri/resources.
                std::env::var("CARGO_MANIFEST_DIR")
                    .ok()
                    .map(|m| PathBuf::from(m).join("resources"))
                    .filter(|d| d.join("geosite.dat").is_file())
            });
        for link in [
            "vless://550e8400-e29b-41d4-a716-446655440000@de-01.zavro.net:443?encryption=none&security=reality&sni=example.com&fp=chrome&pbk=bS9sDoQxkOfKpGVR_9ou_IH1IMj9iz-5E54b_JweDCA&sid=1234abcd&flow=xtls-rprx-vision&type=tcp#DE-01",
            "trojan://pw@nl-02.zavro.net:443#NL-02",
            "vless://550e8400-e29b-41d4-a716-446655440000@fi-01.zavro.net:443?encryption=none&security=tls&sni=fi-01.zavro.net&type=ws&host=fi-01.zavro.net&path=%2Fvless#FI-01",
        ] {
            let node = parse_link(link, "Zavro").unwrap();
            let mut s = settings();
            s.tun_mode = true; // also validate the TUN inbound
            let text =
                build_config_string(&node, &s, &RoutingRules::default(), &PathBuf::from("/tmp"))
                    .unwrap();
            let dir = std::env::temp_dir().join(format!(
                "zavro-xray-test-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            let cfg_path = dir.join("config.json");
            std::fs::write(&cfg_path, text).unwrap();
            crate::xray::test_config(&xray_bin, &cfg_path, asset_dir.as_deref())
                .unwrap_or_else(|e| panic!("xray -test failed for {link}: {e}"));
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}
