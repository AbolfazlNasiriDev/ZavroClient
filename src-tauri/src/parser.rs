//! Real parsers for vless://, vmess://, trojan:// and ss:// links,
//! plus subscription-blob decoding. No mock data: every field comes
//! from the link itself. Unknown/malformed input returns Err.

use crate::types::{new_id, Node};
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use serde_json::Value;

/// Fully-parsed node: everything needed to build an Xray outbound.
#[derive(Debug, Clone)]
pub struct ParsedNode {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub protocol: String, // vless | vmess | trojan | shadowsocks
    pub transport: String,
    pub security: String, // reality | tls | none
    pub group: String,
    pub raw_link: String,
    // credentials / protocol extras
    pub uuid: String,
    pub password: String,
    pub method: String,
    pub alter_id: u16,
    pub sni: String,
    pub fingerprint: String,
    pub public_key: String,
    pub short_id: String,
    pub flow: String,
    pub host: String,
    pub path: String,
    pub service_name: String,
    pub alpn: String,
    /// Shadowsocks plugin options (parsed, not yet mapped to Xray —
    /// Xray has no generic ss-plugin support; kept for round-tripping).
    #[allow(dead_code)]
    pub plugin: String,
}

impl ParsedNode {
    pub fn to_node(&self) -> Node {
        Node {
            id: new_id("node"),
            name: self.name.clone(),
            address: self.address.clone(),
            port: self.port,
            protocol: self.protocol.clone(),
            transport: Some(self.transport.clone()),
            security: Some(self.security.clone()),
            group: Some(self.group.clone()),
            ping_ms: None,
            loss: None,
            speed_mbps: None, // never faked: stays empty until a real speed test exists
            status: Some("idle".to_string()),
            raw_link: Some(self.raw_link.clone()),
        }
    }
}

fn base64_decode_flexible(input: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    // Try as-is with each engine, then with padding restored.
    for engine in [&STANDARD, &URL_SAFE, &STANDARD_NO_PAD, &URL_SAFE_NO_PAD] {
        if let Ok(v) = engine.decode(&cleaned) {
            return Ok(v);
        }
    }
    let mut padded = cleaned.replace('-', "+").replace('_', "/");
    while padded.len() % 4 != 0 {
        padded.push('=');
    }
    STANDARD
        .decode(&padded)
        .map_err(|e| format!("base64 decode failed: {e}"))
}

fn base64_decode_str(input: &str) -> Result<String, String> {
    let bytes = base64_decode_flexible(input)?;
    String::from_utf8(bytes).map_err(|e| format!("decoded bytes are not UTF-8: {e}"))
}

fn query_params(url: &url::Url) -> std::collections::HashMap<String, String> {
    url.query_pairs().into_owned().collect()
}

/// Parse one proxy link into a ParsedNode. `group` is the subscription
/// group name (or "Zavro" / "" for a manually imported link).
pub fn parse_link(link: &str, group: &str) -> Result<ParsedNode, String> {
    let link = link.trim();
    if link.is_empty() {
        return Err("empty link".to_string());
    }
    let scheme_end = link.find("://").ok_or("link has no '://' scheme")?;
    let scheme = link[..scheme_end].to_ascii_lowercase();
    match scheme.as_str() {
        "vless" => parse_vless(link, group),
        "vmess" => parse_vmess(link, group),
        "trojan" => parse_trojan(link, group),
        "ss" => parse_ss(link, group),
        other => Err(format!("unsupported protocol: {other}")),
    }
}

fn parse_vless(link: &str, group: &str) -> Result<ParsedNode, String> {
    let url = url::Url::parse(link).map_err(|e| format!("invalid vless link: {e}"))?;
    let uuid = url.username().to_string();
    if uuid.is_empty() {
        return Err("vless link has no uuid".to_string());
    }
    let address = url.host_str().ok_or("vless link has no host")?.to_string();
    let port = url.port().unwrap_or(443);
    let q = query_params(&url);
    let security = q
        .get("security")
        .cloned()
        .unwrap_or_else(|| "none".to_string());
    let transport = q.get("type").cloned().unwrap_or_else(|| "tcp".to_string());
    let name = url
        .fragment()
        .map(|f| f.to_string())
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| format!("{address}:{port}"));
    Ok(ParsedNode {
        name,
        address,
        port,
        protocol: "vless".to_string(),
        transport,
        security,
        group: group.to_string(),
        raw_link: link.to_string(),
        uuid,
        password: String::new(),
        method: String::new(),
        alter_id: 0,
        sni: q.get("sni").cloned().unwrap_or_default(),
        fingerprint: q.get("fp").cloned().unwrap_or_default(),
        public_key: q.get("pbk").cloned().unwrap_or_default(),
        short_id: q.get("sid").cloned().unwrap_or_default(),
        flow: q.get("flow").cloned().unwrap_or_default(),
        host: q.get("host").cloned().unwrap_or_default(),
        path: q.get("path").cloned().unwrap_or_default(),
        service_name: q.get("serviceName").cloned().unwrap_or_default(),
        alpn: q.get("alpn").cloned().unwrap_or_default(),
        plugin: String::new(),
    })
}

fn parse_trojan(link: &str, group: &str) -> Result<ParsedNode, String> {
    let url = url::Url::parse(link).map_err(|e| format!("invalid trojan link: {e}"))?;
    let password = url.username().to_string();
    if password.is_empty() {
        return Err("trojan link has no password".to_string());
    }
    let address = url.host_str().ok_or("trojan link has no host")?.to_string();
    let port = url.port().unwrap_or(443);
    let q = query_params(&url);
    let sni = q.get("sni").cloned().unwrap_or_else(|| address.clone());
    let name = url
        .fragment()
        .map(|f| f.to_string())
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| format!("{address}:{port}"));
    Ok(ParsedNode {
        name,
        address,
        port,
        protocol: "trojan".to_string(),
        transport: "tcp".to_string(),
        security: "tls".to_string(),
        group: group.to_string(),
        raw_link: link.to_string(),
        uuid: String::new(),
        password,
        method: String::new(),
        alter_id: 0,
        sni,
        fingerprint: String::new(),
        public_key: String::new(),
        short_id: String::new(),
        flow: String::new(),
        host: String::new(),
        path: String::new(),
        service_name: String::new(),
        alpn: q.get("alpn").cloned().unwrap_or_default(),
        plugin: String::new(),
    })
}

fn parse_vmess(link: &str, group: &str) -> Result<ParsedNode, String> {
    let payload = link.strip_prefix("vmess://").ok_or("invalid vmess link")?;
    // A vmess payload may carry a trailing fragment; strip it first.
    let payload = payload.split('#').next().unwrap_or(payload);
    let json_str = base64_decode_str(payload)?;
    let v: Value =
        serde_json::from_str(&json_str).map_err(|e| format!("vmess JSON invalid: {e}"))?;
    let get_s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    let port: u16 = match v.get("port") {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(443) as u16,
        Some(Value::String(s)) => s.parse().unwrap_or(443),
        _ => 443,
    };
    let address = get_s("add");
    if address.is_empty() {
        return Err("vmess JSON has no address (add)".to_string());
    }
    let id = get_s("id");
    if id.is_empty() {
        return Err("vmess JSON has no id".to_string());
    }
    let tls = get_s("tls");
    let security = if tls == "tls" { "tls" } else { "none" }.to_string();
    let name = {
        let ps = get_s("ps");
        if ps.is_empty() {
            format!("{address}:{port}")
        } else {
            ps
        }
    };
    let alter_id: u16 = match v.get("aid") {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0) as u16,
        Some(Value::String(s)) => s.parse().unwrap_or(0),
        _ => 0,
    };
    Ok(ParsedNode {
        name,
        address,
        port,
        protocol: "vmess".to_string(),
        transport: {
            let net = get_s("net");
            if net.is_empty() {
                "tcp".to_string()
            } else {
                net
            }
        },
        security,
        group: group.to_string(),
        raw_link: link.to_string(),
        uuid: id,
        password: String::new(),
        method: String::new(),
        alter_id,
        sni: get_s("sni"),
        fingerprint: get_s("fp"),
        public_key: String::new(),
        short_id: String::new(),
        flow: String::new(),
        host: get_s("host"),
        path: get_s("path"),
        service_name: String::new(),
        alpn: get_s("alpn"),
        plugin: String::new(),
    })
}

fn parse_ss(link: &str, group: &str) -> Result<ParsedNode, String> {
    // Split off fragment (name) and query (plugin opts).
    let (before_frag, frag) = match link.split_once('#') {
        Some((b, f)) => (b, Some(f.to_string())),
        None => (link, None),
    };
    let (before_query, query) = match before_frag.split_once('?') {
        Some((b, q)) => (b, Some(q.to_string())),
        None => (before_frag, None),
    };
    let body = before_query
        .strip_prefix("ss://")
        .ok_or("invalid ss link")?;
    let plugin = query
        .as_deref()
        .and_then(|q| {
            url::form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "plugin")
                .map(|(_, v)| v.into_owned())
        })
        .unwrap_or_default();

    // Resolve (method, password, host, port) from the three encodings.
    let (method, password, address, port) = resolve_ss_body(body)?;
    let name = frag
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| format!("{address}:{port}"));
    Ok(ParsedNode {
        name,
        address,
        port,
        protocol: "shadowsocks".to_string(),
        transport: "tcp".to_string(),
        security: "none".to_string(),
        group: group.to_string(),
        raw_link: link.to_string(),
        uuid: String::new(),
        password,
        method,
        alter_id: 0,
        sni: String::new(),
        fingerprint: String::new(),
        public_key: String::new(),
        short_id: String::new(),
        flow: String::new(),
        host: String::new(),
        path: String::new(),
        service_name: String::new(),
        alpn: String::new(),
        plugin,
    })
}

fn resolve_ss_body(body: &str) -> Result<(String, String, String, u16), String> {
    // Form (b): base64(method:password)@host:port
    if let Some((userinfo_b64, hostport)) = body.split_once('@') {
        if let Ok(decoded) = base64_decode_str(userinfo_b64) {
            if let Some((method, password)) = decoded.split_once(':') {
                // Guard against misreading a plain method (e.g.
                // "aes-256-gcm") as base64: a real method is
                // [A-Za-z0-9._-]+ only.
                let method_ok = !method.is_empty()
                    && method
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "-._".contains(c));
                if method_ok && !password.is_empty() {
                    let (address, port) = parse_hostport(hostport)?;
                    return Ok((method.to_string(), password.to_string(), address, port));
                }
            }
        }
        // Form (c): plain method:password@host:port — url crate handles it.
        if let Ok(url) = url::Url::parse(&format!("ss://{body}")) {
            let method = url.username().to_string();
            let password = url.password().unwrap_or("").to_string();
            if !method.is_empty() {
                if let Some(address) = url.host_str() {
                    return Ok((
                        method,
                        password,
                        address.to_string(),
                        url.port().unwrap_or(8388),
                    ));
                }
            }
        }
        return Err("could not parse ss userinfo".to_string());
    }
    // Form (a): whole body is base64(method:password@host:port)
    if let Ok(decoded) = base64_decode_str(body) {
        if let Some((userinfo, hostport)) = decoded.split_once('@') {
            if let Some((method, password)) = userinfo.split_once(':') {
                let (address, port) = parse_hostport(hostport)?;
                return Ok((method.to_string(), password.to_string(), address, port));
            }
        }
        return Err("base64 ss payload has no method:password@host:port".to_string());
    }
    Err("could not parse ss link body".to_string())
}

fn parse_hostport(hostport: &str) -> Result<(String, u16), String> {
    let hp = hostport.trim();
    match hp.rsplit_once(':') {
        Some((h, p)) if !h.is_empty() => {
            let port: u16 = p.parse().map_err(|_| format!("invalid ss port: {p}"))?;
            Ok((h.trim_matches(|c| c == '[' || c == ']').to_string(), port))
        }
        _ => Err(format!("ss host:port missing port: {hp}")),
    }
}

/// Decode a subscription body: either plain links (one per line) or a
/// base64 blob of links. Returns parsed nodes with `group` applied.
pub fn decode_subscription_blob(body: &str, group: &str) -> Result<Vec<ParsedNode>, String> {
    let text = body.trim();
    if text.is_empty() {
        return Err("subscription is empty".to_string());
    }
    // If it already looks like links, use it directly; otherwise try base64.
    let decoded = if text.contains("://") {
        text.to_string()
    } else {
        base64_decode_str(text)?
    };
    let mut nodes = Vec::new();
    let mut errors = Vec::new();
    for line in decoded.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match parse_link(line, group) {
            Ok(n) => nodes.push(n),
            Err(e) => errors.push(format!("{line}: {e}")),
        }
    }
    if nodes.is_empty() {
        return Err(format!(
            "no valid nodes in subscription{}",
            if errors.is_empty() {
                String::new()
            } else {
                format!("; errors: {}", errors.join("; "))
            }
        ));
    }
    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vless_reality_parses() {
        let link = "vless://550e8400-e29b-41d4-a716-446655440000@de-01.zavro.net:443?encryption=none&security=reality&sni=example.com&fp=chrome&pbk=bS9sDoQxkOfKpGVR_9ou_IH1IMj9iz-5E54b_JweDCA&sid=1234abcd&flow=xtls-rprx-vision&type=tcp#DE-Frankfurt%2001";
        let n = parse_link(link, "Zavro").expect("vless reality should parse");
        assert_eq!(n.protocol, "vless");
        assert_eq!(n.address, "de-01.zavro.net");
        assert_eq!(n.port, 443);
        assert_eq!(n.security, "reality");
        assert_eq!(n.transport, "tcp");
        assert_eq!(n.sni, "example.com");
        assert_eq!(n.fingerprint, "chrome");
        assert_eq!(n.flow, "xtls-rprx-vision");
        assert_eq!(n.public_key, "bS9sDoQxkOfKpGVR_9ou_IH1IMj9iz-5E54b_JweDCA");
        assert_eq!(n.short_id, "1234abcd");
        assert_eq!(n.uuid, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn vless_ws_tls_parses() {
        let link = "vless://550e8400-e29b-41d4-a716-446655440000@fi-01.zavro.net:443?encryption=none&security=tls&sni=fi-01.zavro.net&type=ws&host=fi-01.zavro.net&path=%2Fvless#FI-Helsinki%2001";
        let n = parse_link(link, "Zavro").expect("vless ws should parse");
        assert_eq!(n.transport, "ws");
        assert_eq!(n.security, "tls");
        assert_eq!(n.path, "/vless");
        assert_eq!(n.host, "fi-01.zavro.net");
    }

    #[test]
    fn vless_grpc_parses() {
        let link = "vless://550e8400-e29b-41d4-a716-446655440000@tr-02.zavro.net:443?encryption=none&security=tls&sni=tr-02.zavro.net&type=grpc&serviceName=grpc-service#TR-Istanbul%2002";
        let n = parse_link(link, "Zavro").expect("vless grpc should parse");
        assert_eq!(n.transport, "grpc");
        assert_eq!(n.service_name, "grpc-service");
    }

    #[test]
    fn vmess_base64_json_parses() {
        let json = r#"{"v":"2","ps":"UK-London 01","add":"uk-01.zavro.net","port":"443","id":"550e8400-e29b-41d4-a716-446655440000","aid":"0","net":"ws","type":"none","host":"uk-01.zavro.net","path":"/vmess","tls":"tls","sni":"uk-01.zavro.net"}"#;
        let b64 = STANDARD.encode(json);
        let link = format!("vmess://{b64}");
        let n = parse_link(&link, "Zavro").expect("vmess should parse");
        assert_eq!(n.protocol, "vmess");
        assert_eq!(n.name, "UK-London 01");
        assert_eq!(n.address, "uk-01.zavro.net");
        assert_eq!(n.port, 443);
        assert_eq!(n.transport, "ws");
        assert_eq!(n.security, "tls");
        assert_eq!(n.path, "/vmess");
    }

    #[test]
    fn trojan_parses() {
        let link = "trojan://s3cret-pass@nl-02.zavro.net:443?sni=nl-02.zavro.net#NL-Amsterdam%2002";
        let n = parse_link(link, "Zavro").expect("trojan should parse");
        assert_eq!(n.protocol, "trojan");
        assert_eq!(n.password, "s3cret-pass");
        assert_eq!(n.address, "nl-02.zavro.net");
        assert_eq!(n.security, "tls");
        assert_eq!(n.sni, "nl-02.zavro.net");
    }

    #[test]
    fn ss_base64_whole_parses() {
        // ss:// base64("aes-256-gcm:pass123@sg-01.zavro.net:8388")
        let payload = STANDARD.encode("aes-256-gcm:pass123@sg-01.zavro.net:8388");
        let link = format!("ss://{payload}#SG-Singapore%2001");
        let n = parse_link(&link, "Zavro").expect("ss whole-base64 should parse");
        assert_eq!(n.protocol, "shadowsocks");
        assert_eq!(n.method, "aes-256-gcm");
        assert_eq!(n.password, "pass123");
        assert_eq!(n.address, "sg-01.zavro.net");
        assert_eq!(n.port, 8388);
    }

    #[test]
    fn ss_base64_userinfo_parses() {
        // ss:// base64("aes-128-gcm:pw")@host:port
        let userinfo = URL_SAFE_NO_PAD.encode("aes-128-gcm:pw456");
        let link = format!("ss://{userinfo}@us-03.zavro.net:8388#US-New%20York%2003");
        let n = parse_link(&link, "Zavro").expect("ss userinfo-base64 should parse");
        assert_eq!(n.method, "aes-128-gcm");
        assert_eq!(n.password, "pw456");
        assert_eq!(n.address, "us-03.zavro.net");
    }

    #[test]
    fn ss_plain_parses() {
        let link = "ss://aes-256-gcm:plainpass@sg-01.zavro.net:8388#SG-Plain";
        let n = parse_link(link, "Zavro").expect("ss plain should parse");
        assert_eq!(n.method, "aes-256-gcm");
        assert_eq!(n.password, "plainpass");
    }

    #[test]
    fn subscription_blob_decodes() {
        let links = [
            "vless://550e8400-e29b-41d4-a716-446655440000@de-01.zavro.net:443?security=reality&sni=x.com&type=tcp#DE-01",
            "trojan://pw@nl-02.zavro.net:443#NL-02",
        ];
        let blob = STANDARD.encode(links.join("\n"));
        let nodes = decode_subscription_blob(&blob, "Sub").expect("blob should decode");
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].protocol, "vless");
        assert_eq!(nodes[1].protocol, "trojan");
        // Plain (non-base64) bodies also work.
        let plain = decode_subscription_blob(&links.join("\n"), "Sub").unwrap();
        assert_eq!(plain.len(), 2);
    }

    #[test]
    fn invalid_links_error() {
        assert!(parse_link("", "Zavro").is_err());
        assert!(parse_link("not-a-link", "Zavro").is_err());
        assert!(parse_link("http://example.com", "Zavro").is_err());
        assert!(parse_link("vless://@example.com:443", "Zavro").is_err());
        assert!(decode_subscription_blob("", "Zavro").is_err());
    }
}
