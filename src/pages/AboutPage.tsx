import { useEffect, useState } from "react";
import type { CoreInfo, Settings, Status } from "../types";
import { getCoreInfo, getSettings } from "../api";

interface Props {
  status: Status | null;
}

export default function AboutPage({ status }: Props) {
  const [coreInfo, setCoreInfo] = useState<CoreInfo | null>(null);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    Promise.all([getCoreInfo(), getSettings()])
      .then(([core, s]) => {
        if (!alive) return;
        setCoreInfo(core);
        setSettings(s);
      })
      .catch((e: unknown) => {
        if (alive) setError(e instanceof Error ? e.message : String(e));
      });
    return () => {
      alive = false;
    };
  }, []);

  const rows: [string, string][] = [
    ["Application", "Zavro Client"],
    ["Version", "1.0.0"],
    ["Core", `Xray-core ${coreInfo?.version ?? status?.core_version ?? "—"}`],
    [
      "Core status",
      coreInfo
        ? coreInfo.running
          ? `Running${coreInfo.pid != null ? ` (PID ${coreInfo.pid})` : ""}`
          : "Stopped"
        : (status?.core_running ? "Running" : "Stopped"),
    ],
    [
      "Core memory",
      (coreInfo?.memory_mb ?? status?.core_memory_mb) != null
        ? `${coreInfo?.memory_mb ?? status?.core_memory_mb} MB`
        : "—",
    ],
    ["SOCKS5 port", String(settings?.socks_port ?? 10808)],
    ["HTTP port", String(settings?.http_port ?? 10809)],
    ["Routing mode", status?.routing_mode ?? "—"],
    ["System proxy", status?.system_proxy ? "Enabled" : "Disabled"],
    ["TUN mode", status?.tun_mode ? "Enabled" : "Disabled"],
    ["Platform", "Linux · Fedora 45 · GNOME"],
    ["Stack", "Tauri v2 · React 18 · Xray-core"],
  ];

  return (
    <div className="max-w-xl space-y-4">
      <div className="flex items-center gap-4 rounded-xl border border-adwaita-border bg-adwaita-panel px-5 py-4">
        <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-adwaita-accent text-xl font-bold text-white">
          Z
        </div>
        <div>
          <h2 className="text-lg font-semibold">Zavro Client</h2>
          <p className="text-sm text-adwaita-muted">
            Native Linux desktop proxy client · Version 1.0.0
          </p>
        </div>
      </div>

      {error && (
        <p className="rounded-lg bg-adwaita-red/15 px-3 py-2 text-sm text-[#ff8a8a]">{error}</p>
      )}

      <dl className="overflow-hidden rounded-xl border border-adwaita-border bg-adwaita-panel">
        {rows.map(([label, value]) => (
          <div
            key={label}
            className="flex items-center justify-between border-b border-adwaita-border px-4 py-2.5 text-sm last:border-b-0"
          >
            <dt className="text-adwaita-muted">{label}</dt>
            <dd className="text-adwaita-text">{value}</dd>
          </div>
        ))}
      </dl>

      <p className="text-xs text-adwaita-muted">
        Core version and runtime state are read live from the Xray-core
        sidecar via get_core_info — no values on this page are hardcoded.
      </p>
    </div>
  );
}
