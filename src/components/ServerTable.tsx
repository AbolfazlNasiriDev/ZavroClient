import { Loader2, Trash2, Zap } from "lucide-react";
import type { Node } from "../types";

interface Props {
  nodes: Node[];
  activeNodeId: string | null;
  selectedId: string | null;
  checkingId: string | null;
  onSelect: (node: Node) => void;
  onConnect: (node: Node) => void;
  onDelete: (node: Node) => void;
  onCheckPing: (node: Node) => void;
}

function pingDot(ping: number | null | undefined): string {
  if (ping == null) return "bg-[#777777]";
  if (ping < 80) return "bg-adwaita-green";
  if (ping < 150) return "bg-adwaita-yellow";
  return "bg-adwaita-red";
}

export default function ServerTable({
  nodes,
  activeNodeId,
  selectedId,
  checkingId,
  onSelect,
  onConnect,
  onDelete,
  onCheckPing,
}: Props) {
  if (nodes.length === 0) {
    return (
      <div className="rounded-xl border border-adwaita-border bg-adwaita-panel px-4 py-12 text-center text-sm text-adwaita-muted">
        No servers match. Add a node or a subscription to get started.
      </div>
    );
  }

  return (
    <div className="overflow-hidden rounded-xl border border-adwaita-border bg-adwaita-panel">
      <table className="w-full text-left text-sm">
        <thead>
          <tr className="border-b border-adwaita-border bg-[#333333] text-adwaita-text">
            <th className="px-4 py-3 font-semibold">Name</th>
            <th className="px-4 py-3 font-semibold">Address</th>
            <th className="px-4 py-3 font-semibold">Ping</th>
            <th className="px-4 py-3 font-semibold">Loss</th>
            <th className="px-4 py-3 font-semibold">Speed</th>
            <th className="px-4 py-3 font-semibold">Status</th>
            <th className="px-4 py-3 font-semibold" aria-label="Actions" />
          </tr>
        </thead>
        <tbody>
          {nodes.map((node) => {
            const isActive = node.id === activeNodeId;
            const isSelected = node.id === selectedId;
            return (
              <tr
                key={node.id}
                onClick={() => onSelect(node)}
                onDoubleClick={() => onConnect(node)}
                className={`cursor-pointer border-b border-adwaita-border last:border-b-0 ${
                  isActive
                    ? "bg-[#2f4361]"
                    : isSelected
                      ? "bg-[#3d3d3d]"
                      : "hover:bg-[#404040]"
                }`}
              >
                <td className="px-4 py-3">
                  <div className="flex items-center gap-2.5">
                    <span
                      className={`h-3 w-3 shrink-0 rounded-full ${pingDot(node.ping_ms)}`}
                    />
                    <div className="min-w-0">
                      <div className="flex items-center gap-1.5 font-medium text-adwaita-text">
                        {isActive && (
                          <Zap size={14} className="shrink-0 text-adwaita-yellow" />
                        )}
                        <span className="truncate">{node.name}</span>
                      </div>
                      <div className="text-xs text-adwaita-muted">
                        {node.protocol.toUpperCase()}
                        {node.security ? ` / ${node.security}` : ""}
                        {node.transport ? ` / ${node.transport}` : ""}
                        {node.group ? ` · ${node.group}` : ""}
                      </div>
                    </div>
                  </div>
                </td>
                <td className="px-4 py-3 text-adwaita-text">
                  {node.address}
                  <span className="text-adwaita-muted">:{node.port}</span>
                </td>
                <td className="px-4 py-3 text-adwaita-text">
                  {checkingId === node.id ? (
                    <Loader2 size={15} className="animate-spin text-adwaita-accent" />
                  ) : node.ping_ms != null ? (
                    `${node.ping_ms} ms`
                  ) : (
                    "—"
                  )}
                </td>
                <td className="px-4 py-3 text-adwaita-text">
                  {node.loss != null ? `${node.loss}%` : "—"}
                </td>
                <td className="px-4 py-3 text-adwaita-text">
                  {node.speed_mbps != null
                    ? `${node.speed_mbps.toFixed(1)} Mbps`
                    : "—"}
                </td>
                <td className="px-4 py-3 text-adwaita-text">
                  {isActive ? "Connected" : (node.status ?? "")}
                </td>
                <td className="px-4 py-3">
                  <div
                    className="flex items-center justify-end gap-1"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <button
                      type="button"
                      onClick={() => onCheckPing(node)}
                      disabled={checkingId === node.id}
                      title="Check ping"
                      className="rounded-md px-2 py-1 text-xs text-adwaita-accent hover:bg-adwaita-panel2 disabled:opacity-50"
                    >
                      Ping
                    </button>
                    <button
                      type="button"
                      onClick={() => onConnect(node)}
                      title={isActive ? "Already connected" : "Connect to this node"}
                      className="rounded-md px-2 py-1 text-xs text-adwaita-green hover:bg-adwaita-panel2"
                    >
                      {isActive ? "Active" : "Connect"}
                    </button>
                    <button
                      type="button"
                      onClick={() => onDelete(node)}
                      title="Delete node"
                      aria-label={`Delete ${node.name}`}
                      className="rounded-md p-1.5 text-adwaita-muted hover:bg-adwaita-panel2 hover:text-adwaita-red"
                    >
                      <Trash2 size={15} />
                    </button>
                  </div>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
