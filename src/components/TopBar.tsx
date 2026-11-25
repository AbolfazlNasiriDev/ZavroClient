import { useEffect, useRef, useState } from "react";
import {
  CheckCircle2,
  ChevronDown,
  FileText,
  Link2,
  Plus,
  RefreshCw,
  Settings,
} from "lucide-react";
import type { Status } from "../types";
import { formatBps, routingModeLabel } from "../api";

export type AddNodeTab = "link" | "manual" | "subscription";

interface Props {
  status: Status | null;
  checkingPing: boolean;
  updatingSubs: boolean;
  onToggleConnected: () => void;
  onConnect: () => void;
  onDisconnect: () => void;
  onCheckPing: () => void;
  onUpdateSubscriptions: () => void;
  onOpenAddNode: (tab: AddNodeTab) => void;
  onOpenSettings: () => void;
  onOpenLogs: () => void;
}

export default function TopBar({
  status,
  checkingPing,
  updatingSubs,
  onToggleConnected,
  onConnect,
  onDisconnect,
  onCheckPing,
  onUpdateSubscriptions,
  onOpenAddNode,
  onOpenSettings,
  onOpenLogs,
}: Props) {
  const [menuOpen, setMenuOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const connected = status?.connected ?? false;

  useEffect(() => {
    const close = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setMenuOpen(false);
      }
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, []);

  const openTab = (tab: AddNodeTab) => {
    setMenuOpen(false);
    onOpenAddNode(tab);
  };

  return (
    <header className="border-b border-adwaita-border bg-adwaita-bg px-4 py-3">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <span className="text-sm text-adwaita-text">Connected</span>
          <button
            type="button"
            role="switch"
            aria-checked={connected}
            aria-label="Connected"
            onClick={onToggleConnected}
            className={`relative h-6 w-11 shrink-0 rounded-full transition-colors ${
              connected ? "bg-adwaita-green" : "bg-[#555555]"
            }`}
          >
            <span
              className={`absolute top-0.5 h-5 w-5 rounded-full bg-white shadow transition-all ${
                connected ? "left-[22px]" : "left-0.5"
              }`}
            />
          </button>
          {connected && (
            <span className="flex items-center gap-1 text-xs text-adwaita-green">
              <CheckCircle2 size={14} />
              {status?.active_node_id ? "Active" : "Connected"}
            </span>
          )}
        </div>

        <div className="hidden items-center gap-2 rounded-lg bg-adwaita-panel px-3 py-1.5 text-sm text-adwaita-text md:flex">
          <span>
            Ping:{" "}
            {status?.ping_ms != null ? `${status.ping_ms} ms` : "—"}
          </span>
          <span className="text-adwaita-border">|</span>
          <span>DL: {formatBps(status?.dl_bps ?? 0)}</span>
          <span className="text-adwaita-border">|</span>
          <span>UL: {formatBps(status?.ul_bps ?? 0)}</span>
          <span className="text-adwaita-border">|</span>
          <span>
            Mode: {routingModeLabel(status?.routing_mode ?? "rule")}
          </span>
        </div>
      </div>

      <div className="mt-3 flex flex-wrap items-center gap-2">
        <button
          type="button"
          onClick={onDisconnect}
          disabled={!connected}
          className="rounded-lg bg-adwaita-panel px-4 py-2 text-sm font-medium text-adwaita-text hover:bg-adwaita-panel2 disabled:opacity-40"
        >
          Disconnect
        </button>
        <button
          type="button"
          onClick={onConnect}
          disabled={connected}
          className="rounded-lg bg-adwaita-panel px-4 py-2 text-sm font-medium text-adwaita-text hover:bg-adwaita-panel2 disabled:opacity-40"
        >
          Connect
        </button>

        <div className="relative" ref={menuRef}>
          <button
            type="button"
            onClick={() => setMenuOpen((v) => !v)}
            className="flex items-center gap-2 rounded-lg bg-adwaita-panel px-3 py-2 text-sm text-adwaita-text hover:bg-adwaita-panel2"
          >
            <Plus size={16} />
            Add Node
            <span className="hidden text-adwaita-muted lg:inline">
              VMess, VLESS, Trojan, SS, Subscription
            </span>
            <ChevronDown size={14} className="text-adwaita-muted" />
          </button>
          {menuOpen && (
            <div className="absolute left-0 z-50 mt-1 w-64 overflow-hidden rounded-lg border border-adwaita-border bg-adwaita-panel shadow-xl">
              <button
                type="button"
                onClick={() => openTab("link")}
                className="flex w-full items-center gap-2 px-3 py-2.5 text-left text-sm hover:bg-adwaita-panel2"
              >
                <Link2 size={15} className="text-adwaita-accent" />
                Paste node link (VMess / VLESS / Trojan / SS)
              </button>
              <button
                type="button"
                onClick={() => openTab("manual")}
                className="flex w-full items-center gap-2 px-3 py-2.5 text-left text-sm hover:bg-adwaita-panel2"
              >
                <Plus size={15} className="text-adwaita-accent" />
                Add node manually
              </button>
              <button
                type="button"
                onClick={() => openTab("subscription")}
                className="flex w-full items-center gap-2 px-3 py-2.5 text-left text-sm hover:bg-adwaita-panel2"
              >
                <RefreshCw size={15} className="text-adwaita-accent" />
                Add subscription URL
              </button>
            </div>
          )}
        </div>

        <button
          type="button"
          onClick={onUpdateSubscriptions}
          disabled={updatingSubs}
          className="flex items-center gap-2 rounded-lg bg-adwaita-panel px-3 py-2 text-sm text-adwaita-text hover:bg-adwaita-panel2 disabled:opacity-50"
        >
          <RefreshCw size={15} className={updatingSubs ? "animate-spin" : ""} />
          {updatingSubs ? "Updating…" : "Update Subscriptions"}
        </button>
        <button
          type="button"
          onClick={onCheckPing}
          disabled={checkingPing}
          className="flex items-center gap-2 rounded-lg bg-adwaita-panel px-3 py-2 text-sm text-adwaita-text hover:bg-adwaita-panel2 disabled:opacity-50"
        >
          <CheckCircle2 size={15} className="text-adwaita-green" />
          {checkingPing ? "Checking…" : "Check Ping"}
        </button>
        <button
          type="button"
          onClick={onOpenSettings}
          className="flex items-center gap-2 rounded-lg bg-adwaita-panel px-3 py-2 text-sm text-adwaita-text hover:bg-adwaita-panel2"
        >
          <Settings size={15} />
          Settings
        </button>
        <button
          type="button"
          onClick={onOpenLogs}
          className="flex items-center gap-2 rounded-lg bg-adwaita-panel px-3 py-2 text-sm text-adwaita-text hover:bg-adwaita-panel2"
        >
          <FileText size={15} />
          Logs
        </button>
      </div>
    </header>
  );
}
