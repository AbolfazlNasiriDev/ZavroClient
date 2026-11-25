import {
  FileText,
  Globe,
  Info,
  RefreshCw,
  Settings,
  Cloud,
} from "lucide-react";
import type { PageId, Status } from "../types";
import { routingModeLabel } from "../api";

interface Props {
  page: PageId;
  onNavigate: (page: PageId) => void;
  status: Status | null;
}

const ITEMS: { id: PageId; label: string; icon: typeof Globe }[] = [
  { id: "nodes", label: "Nodes & Servers", icon: Globe },
  { id: "routing", label: "Routing Rules", icon: RefreshCw },
  { id: "subscriptions", label: "Subscriptions", icon: Cloud },
  { id: "proxy", label: "Proxy Settings", icon: Settings },
  { id: "logs", label: "System Logs", icon: FileText },
  { id: "about", label: "About", icon: Info },
];

export default function Sidebar({ page, onNavigate, status }: Props) {
  return (
    <aside className="flex w-56 shrink-0 flex-col border-r border-adwaita-border bg-adwaita-bg">
      <nav className="flex-1 space-y-1 p-3">
        {ITEMS.map((item) => {
          const Icon = item.icon;
          const active = page === item.id;
          return (
            <button
              key={item.id}
              type="button"
              onClick={() => onNavigate(item.id)}
              className={`flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left text-sm transition-colors ${
                active
                  ? "bg-adwaita-panel font-medium text-adwaita-text"
                  : "text-adwaita-muted hover:bg-adwaita-panel hover:text-adwaita-text"
              }`}
            >
              <Icon size={17} />
              {item.label}
            </button>
          );
        })}
      </nav>

      <div className="space-y-1.5 border-t border-adwaita-border px-4 py-4 text-[13px] text-adwaita-muted">
        <p className="flex items-center gap-2">
          <span
            className={`h-2 w-2 rounded-full ${
              status?.connected ? "bg-adwaita-green" : "bg-adwaita-muted"
            }`}
          />
          Active Profile: Default
        </p>
        <p className="flex items-center gap-2">
          <span
            className={`h-2 w-2 rounded-full ${
              status?.core_running ? "bg-adwaita-green" : "bg-adwaita-muted"
            }`}
          />
          Core: Xray-core {status?.core_version ?? "—"}
        </p>
        <p className="flex items-center gap-2">
          <span className="h-2 w-2 rounded-full bg-adwaita-green" />
          Mode: {status?.tun_mode
            ? "TUN Mode (Enabled)"
            : routingModeLabel(status?.routing_mode ?? "rule")}
        </p>
      </div>
    </aside>
  );
}
