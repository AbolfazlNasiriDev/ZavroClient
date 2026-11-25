export type Protocol = "vless" | "vmess" | "trojan" | "shadowsocks";

export interface Node {
  id: string;
  name: string;
  address: string;
  port: number;
  protocol: Protocol;
  transport?: string;
  security?: string;
  group?: string;
  ping_ms?: number | null;
  loss?: number | null;
  speed_mbps?: number | null;
  status?: string;
  raw_link?: string;
}

export interface Settings {
  socks_port: number;
  http_port: number;
  routing_mode: string;
  dns: string[];
  auto_update_hours: number;
  system_proxy: boolean;
  tun_mode: boolean;
  log_level: string;
}

export interface Subscription {
  id: string;
  url: string;
  name: string;
  node_count: number;
  last_updated?: string | null;
}

export interface Status {
  connected: boolean;
  active_node_id: string | null;
  ping_ms: number | null;
  dl_bps: number;
  ul_bps: number;
  routing_mode: string;
  system_proxy: boolean;
  tun_mode: boolean;
  core_running: boolean;
  core_pid: number | null;
  core_memory_mb: number | null;
  core_version: string;
}

export interface PingResult {
  node_id: string;
  ping_ms: number | null;
  loss: number;
}

export interface LogEntry {
  ts: string;
  level: string;
  msg: string;
}

export interface TrafficPoint {
  t: number;
  dl: number;
  ul: number;
}

export interface Traffic {
  dl_bps: number;
  ul_bps: number;
  points: TrafficPoint[];
}

export interface CoreInfo {
  version: string;
  running: boolean;
  pid: number | null;
  memory_mb: number | null;
}

export interface RoutingRules {
  mode: string;
  bypass_domains: string[];
  bypass_ips: string[];
}

export type PageId =
  | "nodes"
  | "routing"
  | "subscriptions"
  | "proxy"
  | "logs"
  | "about";
