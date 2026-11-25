import { useCallback, useEffect, useState } from "react";
import { Plus, RefreshCw, Trash2 } from "lucide-react";
import type { Node, Subscription } from "../types";
import {
  addSubscription,
  deleteSubscription,
  getSubscriptions,
  importSubscription,
  updateSubscriptions,
} from "../api";

interface Props {
  onNodesChanged: (nodes: Node[]) => void;
}

export default function SubscriptionsPage({ onNodesChanged }: Props) {
  const [subs, setSubs] = useState<Subscription[]>([]);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState<string | null>(null);
  const [url, setUrl] = useState("");

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setSubs(await getSubscriptions());
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const handleAdd = async () => {
    const value = url.trim();
    if (!value) {
      setError("Paste a subscription URL first.");
      return;
    }
    setBusy(true);
    setError(null);
    setDone(null);
    try {
      const sub = await addSubscription(value);
      const nodes = await importSubscription(value);
      onNodesChanged(nodes);
      setSubs((prev) => [...prev, sub]);
      setDone(`Added subscription — ${nodes.length} node(s) imported.`);
      setUrl("");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleUpdateAll = async () => {
    setBusy(true);
    setError(null);
    setDone(null);
    try {
      const nodes = await updateSubscriptions();
      onNodesChanged(nodes);
      await refresh();
      setDone(`Subscriptions updated — ${nodes.length} node(s) in total.`);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleDelete = async (sub: Subscription) => {
    setError(null);
    try {
      await deleteSubscription(sub.id);
      setSubs((prev) => prev.filter((s) => s.id !== sub.id));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div className="max-w-3xl space-y-4">
      <div className="flex flex-wrap gap-2">
        <input
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") void handleAdd();
          }}
          placeholder="https://example.com/sub/…"
          aria-label="New subscription URL"
          className="min-w-[260px] flex-1 rounded-lg border border-adwaita-border bg-adwaita-panel px-3 py-2 font-mono text-xs text-adwaita-text placeholder:text-adwaita-muted"
        />
        <button
          type="button"
          onClick={() => void handleAdd()}
          disabled={busy}
          className="flex items-center gap-1.5 rounded-lg bg-adwaita-accent px-4 py-2 text-sm font-medium text-white hover:bg-adwaita-accentHover disabled:opacity-50"
        >
          <Plus size={15} /> Add Subscription
        </button>
        <button
          type="button"
          onClick={() => void handleUpdateAll()}
          disabled={busy}
          className="flex items-center gap-1.5 rounded-lg bg-adwaita-panel px-4 py-2 text-sm text-adwaita-text hover:bg-adwaita-panel2 disabled:opacity-50"
        >
          <RefreshCw size={15} className={busy ? "animate-spin" : ""} />
          Update All
        </button>
      </div>

      {error && (
        <p className="rounded-lg bg-adwaita-red/15 px-3 py-2 text-sm text-[#ff8a8a]">{error}</p>
      )}
      {done && (
        <p className="rounded-lg bg-adwaita-green/15 px-3 py-2 text-sm text-adwaita-green">{done}</p>
      )}

      {loading ? (
        <p className="text-sm text-adwaita-muted">Loading subscriptions…</p>
      ) : subs.length === 0 ? (
        <div className="rounded-xl border border-adwaita-border bg-adwaita-panel px-4 py-10 text-center text-sm text-adwaita-muted">
          No subscriptions yet. Paste a subscription URL above to import its
          nodes.
        </div>
      ) : (
        <div className="overflow-hidden rounded-xl border border-adwaita-border bg-adwaita-panel">
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="border-b border-adwaita-border bg-[#333333]">
                <th className="px-4 py-3 font-semibold">Name / URL</th>
                <th className="px-4 py-3 font-semibold">Nodes</th>
                <th className="px-4 py-3 font-semibold">Last updated</th>
                <th className="px-4 py-3" aria-label="Actions" />
              </tr>
            </thead>
            <tbody>
              {subs.map((sub) => (
                <tr key={sub.id} className="border-b border-adwaita-border last:border-b-0">
                  <td className="px-4 py-3">
                    <div className="font-medium">{sub.name || "Subscription"}</div>
                    <div className="max-w-md truncate font-mono text-xs text-adwaita-muted">
                      {sub.url}
                    </div>
                  </td>
                  <td className="px-4 py-3">{sub.node_count}</td>
                  <td className="px-4 py-3 text-adwaita-muted">
                    {sub.last_updated ?? "—"}
                  </td>
                  <td className="px-4 py-3 text-right">
                    <button
                      type="button"
                      onClick={() => void handleDelete(sub)}
                      aria-label={`Delete subscription ${sub.name || sub.url}`}
                      title="Delete subscription"
                      className="rounded-md p-1.5 text-adwaita-muted hover:bg-adwaita-panel2 hover:text-adwaita-red"
                    >
                      <Trash2 size={15} />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
