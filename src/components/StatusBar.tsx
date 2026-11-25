import type { CoreInfo, Settings, Status } from "../types";

interface Props {
  status: Status | null;
  coreInfo: CoreInfo | null;
  settings: Settings | null;
}

export default function StatusBar({ status, coreInfo, settings }: Props) {
  const running = coreInfo?.running ?? status?.core_running ?? false;
  const memory = coreInfo?.memory_mb ?? status?.core_memory_mb;
  const socks = settings?.socks_port ?? 10808;
  const http = settings?.http_port ?? 10809;

  return (
    <footer className="flex flex-wrap items-center justify-center gap-x-3 gap-y-1 border-t border-adwaita-border bg-adwaita-bg px-4 py-2 text-[13px] text-adwaita-muted">
      <span>
        Core: Xray ({running ? "Running" : "Stopped"})
        {status?.core_pid != null ? ` · PID ${status.core_pid}` : ""}
      </span>
      <span className="text-adwaita-border">|</span>
      <span>Memory: {memory != null ? `${memory} MB` : "—"}</span>
      <span className="text-adwaita-border">|</span>
      <span>
        Port: {socks} (SOCKS5), {http} (HTTP)
      </span>
    </footer>
  );
}
