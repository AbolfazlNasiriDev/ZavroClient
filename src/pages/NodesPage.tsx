import { useMemo, useState } from "react";
import { ArrowUpDown, Search } from "lucide-react";
import type { Node, Status } from "../types";
import ServerTable from "../components/ServerTable";

interface Props {
  nodes: Node[];
  status: Status | null;
  selectedId: string | null;
  checkingId: string | null;
  onSelect: (node: Node) => void;
  onConnect: (node: Node) => void;
  onDelete: (node: Node) => void;
  onCheckPing: (node: Node) => void;
  onToggleSystemProxy: (enabled: boolean) => void;
  onToggleTun: (enabled: boolean) => void;
}

type SortKey = "ping" | "name" | "speed";

export default function NodesPage({
  nodes,
  status,
  selectedId,
  checkingId,
  onSelect,
  onConnect,
  onDelete,
  onCheckPing,
  onToggleSystemProxy,
  onToggleTun,
}: Props) {
  const [search, setSearch] = useState("");
  const [group, setGroup] = useState("All");
  const [sort, setSort] = useState<SortKey>("ping");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");

  const groups = useMemo(() => {
    const set = new Set<string>();
    nodes.forEach((n) => {
      if (n.group) set.add(n.group);
    });
    return ["All", ...[...set].sort()];
  }, [nodes]);

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    const list = nodes.filter((n) => {
      if (group !== "All" && n.group !== group) return false;
      if (!q) return true;
      return (
        n.name.toLowerCase().includes(q) ||
        n.address.toLowerCase().includes(q) ||
        n.protocol.toLowerCase().includes(q) ||
        (n.group ?? "").toLowerCase().includes(q)
      );
    });
    const dir = sortDir === "asc" ? 1 : -1;
    return [...list].sort((a, b) => {
      if (sort === "name") return dir * a.name.localeCompare(b.name);
      if (sort === "ping") {
        const pa = a.ping_ms ?? Number.POSITIVE_INFINITY;
        const pb = b.ping_ms ?? Number.POSITIVE_INFINITY;
        return dir * (pa - pb);
      }
      const sa = a.speed_mbps ?? -1;
      const sb = b.speed_mbps ?? -1;
      return dir * (sa - sb);
    });
  }, [nodes, search, group, sort, sortDir]);

  const toggleSort = (key: SortKey) => {
    if (sort === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSort(key);
      setSortDir("asc");
    }
  };

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-3">
        <div className="relative min-w-[220px] flex-1">
          <Search
            size={16}
            className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-adwaita-muted"
          />
          <input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search servers..."
            aria-label="Search servers"
            className="w-full rounded-lg border border-adwaita-border bg-adwaita-panel py-2 pl-9 pr-3 text-sm text-adwaita-text placeholder:text-adwaita-muted"
          />
        </div>

        <label className="flex items-center gap-2 text-sm text-adwaita-muted">
          Group:
          <select
            value={group}
            onChange={(e) => setGroup(e.target.value)}
            aria-label="Filter by group"
            className="rounded-lg border border-adwaita-border bg-adwaita-panel px-2 py-2 text-sm text-adwaita-text"
          >
            {groups.map((g) => (
              <option key={g} value={g}>
                {g}
              </option>
            ))}
          </select>
        </label>

        <label className="flex items-center gap-2 text-sm text-adwaita-muted">
          Sort:
          <select
            value={sort}
            onChange={(e) => setSort(e.target.value as SortKey)}
            aria-label="Sort by"
            className="rounded-lg border border-adwaita-border bg-adwaita-panel px-2 py-2 text-sm capitalize text-adwaita-text"
          >
            <option value="ping">Ping</option>
            <option value="name">Name</option>
            <option value="speed">Speed</option>
          </select>
          <button
            type="button"
            onClick={() => toggleSort(sort)}
            title="Toggle sort direction"
            aria-label="Toggle sort direction"
            className="rounded-lg border border-adwaita-border bg-adwaita-panel p-2 text-adwaita-text hover:bg-adwaita-panel2"
          >
            <ArrowUpDown
              size={15}
              className={sortDir === "desc" ? "rotate-180" : ""}
            />
          </button>
        </label>

        <label className="flex items-center gap-2 text-sm text-adwaita-text">
          System Proxy
          <button
            type="button"
            role="switch"
            aria-checked={status?.system_proxy ?? false}
            aria-label="System Proxy"
            onClick={() =>
              onToggleSystemProxy(!(status?.system_proxy ?? false))
            }
            className={`relative h-6 w-11 rounded-full transition-colors ${
              status?.system_proxy ? "bg-adwaita-accent" : "bg-[#555555]"
            }`}
          >
            <span
              className={`absolute top-0.5 h-5 w-5 rounded-full bg-white shadow transition-all ${
                status?.system_proxy ? "left-[22px]" : "left-0.5"
              }`}
            />
          </button>
        </label>

        <label className="flex items-center gap-2 text-sm text-adwaita-text">
          TUN Mode
          <button
            type="button"
            role="switch"
            aria-checked={status?.tun_mode ?? false}
            aria-label="TUN Mode"
            onClick={() => onToggleTun(!(status?.tun_mode ?? false))}
            className={`relative h-6 w-11 rounded-full transition-colors ${
              status?.tun_mode ? "bg-adwaita-accent" : "bg-[#555555]"
            }`}
          >
            <span
              className={`absolute top-0.5 h-5 w-5 rounded-full bg-white shadow transition-all ${
                status?.tun_mode ? "left-[22px]" : "left-0.5"
              }`}
            />
          </button>
        </label>
      </div>

      <p className="text-xs text-adwaita-muted">
        {filtered.length} of {nodes.length} server(s)
        {status?.active_node_id
          ? " · double-click a row to connect"
          : " · double-click a row to connect"}
      </p>

      <ServerTable
        nodes={filtered}
        activeNodeId={status?.active_node_id ?? null}
        selectedId={selectedId}
        checkingId={checkingId}
        onSelect={onSelect}
        onConnect={onConnect}
        onDelete={onDelete}
        onCheckPing={onCheckPing}
      />
    </div>
  );
}
