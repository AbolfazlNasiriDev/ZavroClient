import { useEffect, useState } from "react";
import { RefreshCw, Trash2, X } from "lucide-react";
import type { LogEntry } from "../types";
import { clearLogs, getLogs } from "../api";

interface Props {
  onClose: () => void;
}

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

export default function LogsModal({ onClose }: Props) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    setLoading(true);
    setError(null);
    try {
      setLogs(await getLogs());
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleClear = async () => {
    try {
      await clearLogs();
      setLogs([]);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      onClick={onClose}
    >
      <div
        className="flex max-h-[80vh] w-full max-w-2xl flex-col rounded-2xl border border-adwaita-border bg-adwaita-bg shadow-2xl"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-label="Logs"
      >
        <div className="flex items-center justify-between border-b border-adwaita-border px-5 py-4">
          <h2 className="text-base font-semibold">Logs</h2>
          <div className="flex items-center gap-1">
            <button
              type="button"
              onClick={() => void refresh()}
              title="Refresh logs"
              className="rounded-md p-1.5 text-adwaita-muted hover:bg-adwaita-panel hover:text-adwaita-text"
            >
              <RefreshCw size={16} />
            </button>
            <button
              type="button"
              onClick={() => void handleClear()}
              title="Clear logs"
              className="rounded-md p-1.5 text-adwaita-muted hover:bg-adwaita-panel hover:text-adwaita-red"
            >
              <Trash2 size={16} />
            </button>
            <button
              type="button"
              onClick={onClose}
              aria-label="Close"
              className="rounded-md p-1 text-adwaita-muted hover:bg-adwaita-panel hover:text-adwaita-text"
            >
              <X size={18} />
            </button>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto px-5 py-3 font-mono text-xs leading-6">
          {loading && <p className="text-adwaita-muted">Loading logs…</p>}
          {error && <p className="text-[#ff8a8a]">{error}</p>}
          {!loading && !error && logs.length === 0 && (
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
    </div>
  );
}
