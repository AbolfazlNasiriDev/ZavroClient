//! Persistence: a single JSON file (`zavro.json`) in the Tauri
//! app-data dir. Nodes, subscriptions, settings, routing rules.
//! On first run it is seeded ONLY with the 7 reference nodes from the
//! project brief — real addresses/protocols, EMPTY ping/speed
//! (never faked measurements).

use crate::parser::parse_link;
use crate::types::{Node, RoutingRules, Settings, Subscription};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreData {
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub subscriptions: Vec<Subscription>,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub routing: RoutingRules,
}

impl Default for StoreData {
    fn default() -> Self {
        Self {
            nodes: seed_nodes(),
            subscriptions: Vec::new(),
            settings: Settings::default(),
            routing: RoutingRules::default(),
        }
    }
}

pub fn store_path(data_dir: &Path) -> PathBuf {
    data_dir.join("zavro.json")
}

pub fn load(data_dir: &Path) -> StoreData {
    let path = store_path(data_dir);
    match std::fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<StoreData>(&text) {
            Ok(mut data) => {
                // Keep stored nodes even if empty (user may have deleted
                // them all); only a corrupt/missing file re-seeds.
                if data.settings.dns.is_empty() {
                    data.settings.dns = Settings::default().dns;
                }
                data
            }
            Err(_) => StoreData::default(),
        },
        Err(_) => StoreData::default(),
    }
}

pub fn save(data_dir: &Path, data: &StoreData) -> Result<(), String> {
    std::fs::create_dir_all(data_dir).map_err(|e| format!("cannot create app data dir: {e}"))?;
    let text =
        serde_json::to_string_pretty(data).map_err(|e| format!("cannot serialize store: {e}"))?;
    std::fs::write(store_path(data_dir), text).map_err(|e| format!("cannot write store: {e}"))?;
    Ok(())
}

/// The 7 reference nodes. Their raw links are well-formed but carry
/// placeholder credentials — the user replaces them with real links.
/// ping/speed are left empty (None) on purpose: no fake measurements.
pub fn seed_nodes() -> Vec<Node> {
    // (name, link) — uuid/password values are placeholders to be
    // replaced by the user; the addresses, ports and protocols are
    // the real reference topology.
    let seeds: [(&str, &str); 7] = [
        (
            "DE-Frankfurt 01",
            "vless://00000000-0000-4000-8000-000000000000@de-01.zavro.net:443?encryption=none&security=reality&sni=de-01.zavro.net&fp=chrome&type=tcp#DE-Frankfurt%2001",
        ),
        (
            "NL-Amsterdam 02",
            "trojan://REPLACE-ME@nl-02.zavro.net:443?sni=nl-02.zavro.net#NL-Amsterdam%2002",
        ),
        (
            "UK-London 01",
            "vmess://eyJ2IjoiMiIsInBzIjoiVUstTG9uZG9uIDAxIiwiYWRkIjoidWstMDEuemF2cm8ubmV0IiwicG9ydCI6IjQ0MyIsImlkIjoiMDAwMDAwMDAtMDAwMC00MDAwLTgwMDAtMDAwMDAwMDAwMDAwIiwiYWlkIjoiMCIsIm5ldCI6IndzIiwidHlwZSI6Im5vbmUiLCJob3N0IjoidWstMDEuemF2cm8ubmV0IiwicGF0aCI6Ii92bWVzcyIsInRscyI6InRscyIsInNuaSI6InVrLTAxLnphdnJvLm5ldCJ9",
        ),
        (
            "FI-Helsinki 01",
            "vless://00000000-0000-4000-8000-000000000000@fi-01.zavro.net:443?encryption=none&security=tls&sni=fi-01.zavro.net&type=ws&host=fi-01.zavro.net&path=%2Fvless#FI-Helsinki%2001",
        ),
        (
            "US-New York 03",
            "trojan://REPLACE-ME@us-03.zavro.net:443?sni=us-03.zavro.net#US-New%20York%2003",
        ),
        (
            "SG-Singapore 01",
            "vmess://eyJ2IjoiMiIsInBzIjoiU0ctU2luZ2Fwb3JlIDAxIiwiYWRkIjoic2ctMDEuemF2cm8ubmV0IiwicG9ydCI6IjQ0MyIsImlkIjoiMDAwMDAwMDAtMDAwMC00MDAwLTgwMDAtMDAwMDAwMDAwMDAwIiwiYWlkIjoiMCIsIm5ldCI6IndzIiwidHlwZSI6Im5vbmUiLCJob3N0Ijoic2ctMDEuemF2cm8ubmV0IiwicGF0aCI6Ii92bWVzcyIsInRscyI6InRscyIsInNuaSI6InNnLTAxLnphdnJvLm5ldCJ9",
        ),
        (
            "TR-Istanbul 02",
            "vless://00000000-0000-4000-8000-000000000000@tr-02.zavro.net:443?encryption=none&security=reality&sni=tr-02.zavro.net&fp=chrome&type=tcp#TR-Istanbul%2002",
        ),
    ];
    seeds
        .iter()
        .map(|(name, link)| {
            let mut node = parse_link(link, "Zavro")
                .expect("seed link must parse")
                .to_node();
            node.name = name.to_string();
            node
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_has_seven_reference_nodes_with_empty_measurements() {
        let nodes = seed_nodes();
        assert_eq!(nodes.len(), 7);
        let addrs: Vec<&str> = nodes.iter().map(|n| n.address.as_str()).collect();
        assert_eq!(
            addrs,
            [
                "de-01.zavro.net",
                "nl-02.zavro.net",
                "uk-01.zavro.net",
                "fi-01.zavro.net",
                "us-03.zavro.net",
                "sg-01.zavro.net",
                "tr-02.zavro.net",
            ]
        );
        assert_eq!(
            nodes
                .iter()
                .map(|n| n.protocol.as_str())
                .collect::<Vec<_>>(),
            ["vless", "trojan", "vmess", "vless", "trojan", "vmess", "vless"]
        );
        assert_eq!(
            nodes.iter().map(|n| n.port).collect::<Vec<_>>(),
            [443, 443, 443, 443, 443, 443, 443]
        );
        assert!(nodes.iter().all(|n| n.group.as_deref() == Some("Zavro")));
        // No fake measurements.
        assert!(nodes.iter().all(|n| n.ping_ms.is_none()));
        assert!(nodes.iter().all(|n| n.speed_mbps.is_none()));
    }
}
