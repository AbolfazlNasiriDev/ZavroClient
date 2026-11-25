import { invoke } from "@tauri-apps/api/core";
import type {
  CoreInfo,
  LogEntry,
  Node,
  PingResult,
  RoutingRules,
  Settings,
  Status,
  Subscription,
  Traffic,
} from "./types";

export async function getStatus(): Promise<Status> {
  return invoke<Status>("get_status");
}

export async function listNodes(): Promise<Node[]> {
  return invoke<Node[]>("list_nodes");
}

export async function addNode(node: Node): Promise<Node> {
  return invoke<Node>("add_node", { node });
}

export async function updateNode(node: Node): Promise<Node> {
  return invoke<Node>("update_node", { node });
}

export async function deleteNode(id: string): Promise<void> {
  return invoke<void>("delete_node", { id });
}

export async function importLink(link: string): Promise<Node> {
  return invoke<Node>("import_link", { link });
}

export async function importSubscription(url: string): Promise<Node[]> {
  return invoke<Node[]>("import_subscription", { url });
}

export async function updateSubscriptions(): Promise<Node[]> {
  return invoke<Node[]>("update_subscriptions");
}

export async function getSubscriptions(): Promise<Subscription[]> {
  return invoke<Subscription[]>("get_subscriptions");
}

export async function addSubscription(url: string): Promise<Subscription> {
  return invoke<Subscription>("add_subscription", { url });
}

export async function deleteSubscription(id: string): Promise<void> {
  return invoke<void>("delete_subscription", { id });
}

export async function checkPingAll(): Promise<PingResult[]> {
  return invoke<PingResult[]>("check_ping_all");
}

export async function checkPing(id: string): Promise<PingResult> {
  return invoke<PingResult>("check_ping", { id });
}

export async function connect(nodeId: string): Promise<void> {
  // Rust parameter is `node_id` (snake_case) — Tauri deserialises by exact name.
  return invoke<void>("connect", { node_id: nodeId });
}

export async function disconnect(): Promise<void> {
  return invoke<void>("disconnect");
}

export async function setSystemProxy(enabled: boolean): Promise<void> {
  return invoke<void>("set_system_proxy", { enabled });
}

export async function setTun(enabled: boolean): Promise<void> {
  return invoke<void>("set_tun", { enabled });
}

export async function setRoutingMode(mode: string): Promise<void> {
  return invoke<void>("set_routing_mode", { mode });
}

export async function getLogs(): Promise<LogEntry[]> {
  return invoke<LogEntry[]>("get_logs");
}

export async function clearLogs(): Promise<void> {
  return invoke<void>("clear_logs");
}

export async function getTraffic(): Promise<Traffic> {
  return invoke<Traffic>("get_traffic");
}

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export async function saveSettings(s: Settings): Promise<void> {
  return invoke<void>("save_settings", { s });
}

export async function getCoreInfo(): Promise<CoreInfo> {
  return invoke<CoreInfo>("get_core_info");
}

export async function getRoutingRules(): Promise<RoutingRules> {
  return invoke<RoutingRules>("get_routing_rules");
}

export async function saveRoutingRules(r: RoutingRules): Promise<void> {
  return invoke<void>("save_routing_rules", { r });
}

/** Human-readable throughput, e.g. 3.4 MB/s, 184 KB/s, 0 B/s. */
export function formatBps(bps: number): string {
  if (!bps || bps <= 0) return "0 B/s";
  if (bps >= 1_000_000) return `${(bps / 1_000_000).toFixed(1)} MB/s`;
  if (bps >= 1_000) return `${(bps / 1_000).toFixed(0)} KB/s`;
  return `${bps.toFixed(0)} B/s`;
}

/** Human-readable routing mode label. */
export function routingModeLabel(mode: string): string {
  switch (mode) {
    case "rule":
      return "Rule (PAC)";
    case "global":
      return "Global";
    case "direct":
      return "Direct";
    case "tun":
      return "TUN Mode";
    default:
      return mode;
  }
}
