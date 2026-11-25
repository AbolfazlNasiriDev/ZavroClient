import { useCallback, useEffect, useState } from "react";
import { RefreshCw, Trash2 } from "lucide-react";
import type { LogEntry } from "../types";
import { clearLogs, getLogs } from "../api";

const levelColor = (level: string): string => {
  switch (level.toLowerCase()) {
    case "error":
      return "text-[#ff8a8a]";
    case "warn":
      return "text-adwaita-yellow";
    case "debug":
      return "text-adwaita-muted";
    default:
      return "text-adwaita-text";
  }
};

export default function LogsPage() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setError(null);
    try {
      setLogs(await getLogs());
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
    const id = window.setInterval(() => void refresh(), 5000);
    return () => window.clearInterval(id);
  }, [refresh]);

  const handleClear = async () => {
    try {
      await clearLogs();
      setLogs([]);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div className="flex h-full flex-col space-y-3">
      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={() => void refresh()}
          className="flex items-center gap-1.5 rounded-lg bg-adwaita-panel px-3 py-2 text-sm text-adwaita-text hover:bg-adwaita-panel2"
        >
          <RefreshCw size={15} /> Refresh
        </button>
        <button
          type="button"
          onClick={() => void handleClear()}
          className="flex items-center gap-1.5 rounded-lg bg-adwaita-panel px-3 py-2 text-sm text-adwaita-text hover:bg-adwaita-panel2"
        >
          <Trash2 size={15} /> Clear Logs
        </button>
        <span className="text-xs text-adwaita-muted">Auto-refreshes every 5 s</span>
      </div>

      {error && (
        <p className="rounded-lg bg-adwaita-red/15 px-3 py-2 text-sm text-[#ff8a8a]">{error}</p>
      )}

      <div className="flex-1 overflow-y-auto rounded-xl border border-adwaita-border bg-adwaita-panel px-4 py-3 font-mono text-xs leading-6">
        {loading && <p className="text-adwaita-muted">Loading logs…</p>}
        {!loading && logs.length === 0 && (
          <p className="text-adwaita-muted">No log entries yet.</p>
        )}
        {logs.map((entry, i) => (
          <p key={`${entry.ts}-${i}`} className="break-words">
            <span className="text-adwaita-muted">{entry.ts}</span>{" "}
            <span className={`font-semibold ${levelColor(entry.level)}`}>
              [{entry.level.toUpperCase()}]
            </span>{" "}
            <span className="text-adwaita-text">{entry.msg}</span>
          </p>
        ))}
      </div>
    </div>
  );
}
