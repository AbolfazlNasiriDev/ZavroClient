import { useCallback, useEffect, useRef, useState } from "react";
import type {
  CoreInfo,
  Node,
  PageId,
  Settings,
  Status,
  Traffic,
} from "./types";
import {
  checkPing,
  checkPingAll,
  connect,
  disconnect,
  getCoreInfo,
  getSettings,
  getStatus,
  getTraffic,
  listNodes,
  deleteNode,
  setSystemProxy,
  setTun,
  updateSubscriptions,
} from "./api";
import TopBar, { type AddNodeTab } from "./components/TopBar";
import Sidebar from "./components/Sidebar";
import TrafficChart from "./components/TrafficChart";
import StatusBar from "./components/StatusBar";
import AddNodeModal from "./components/AddNodeModal";
import SettingsModal from "./components/SettingsModal";
import LogsModal from "./components/LogsModal";
import NodesPage from "./pages/NodesPage";
import RoutingPage from "./pages/RoutingPage";
import SubscriptionsPage from "./pages/SubscriptionsPage";
import ProxySettingsPage from "./pages/ProxySettingsPage";
import LogsPage from "./pages/LogsPage";
import AboutPage from "./pages/AboutPage";

const PAGE_TITLES: Record<PageId, string> = {
  nodes: "Nodes & Servers",
  routing: "Routing Rules",
  subscriptions: "Subscriptions",
  proxy: "Proxy Settings",
  logs: "System Logs",
  about: "About",
};

function mergeNodes(prev: Node[], incoming: Node[]): Node[] {
  const map = new Map(prev.map((n) => [n.id, n]));
  incoming.forEach((n) => map.set(n.id, { ...map.get(n.id), ...n }));
  return [...map.values()];
}

export default function App() {
  const [page, setPage] = useState<PageId>("nodes");
  const [status, setStatus] = useState<Status | null>(null);
  const [nodes, setNodes] = useState<Node[]>([]);
  const [traffic, setTraffic] = useState<Traffic | null>(null);
  const [coreInfo, setCoreInfo] = useState<CoreInfo | null>(null);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [checkingPing, setCheckingPing] = useState(false);
  const [checkingId, setCheckingId] = useState<string | null>(null);
  const [updatingSubs, setUpdatingSubs] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [addNodeTab, setAddNodeTab] = useState<AddNodeTab | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [logsOpen, setLogsOpen] = useState(false);
  const nodesRef = useRef<Node[]>([]);
  nodesRef.current = nodes;
  const selectedRef = useRef<string | null>(null);
  selectedRef.current = selectedId;

  const refreshStatus = useCallback(async () => {
    try {
      setStatus(await getStatus());
    } catch {
      // Backend not reachable yet (e.g. running outside Tauri); keep last state.
    }
  }, []);

  const refreshNodes = useCallback(async () => {
    try {
      setNodes(await listNodes());
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  const refreshTraffic = useCallback(async () => {
    try {
      setTraffic(await getTraffic());
    } catch {
      // Ignore transient traffic errors; chart keeps last points.
    }
  }, []);

  useEffect(() => {
    void refreshStatus();
    void refreshNodes();
    void refreshTraffic();
    getCoreInfo().then(setCoreInfo).catch(() => undefined);
    getSettings().then(setSettings).catch(() => undefined);
    const id = window.setInterval(() => {
      void refreshStatus();
      void refreshTraffic();
    }, 2000);
    return () => window.clearInterval(id);
  }, [refreshStatus, refreshNodes, refreshTraffic]);

  const fail = (e: unknown) => {
    setError(e instanceof Error ? e.message : String(e));
  };

  const handleToggleConnected = async () => {
    setError(null);
    try {
      if (status?.connected) {
        await disconnect();
      } else {
        const target =
          nodesRef.current.find((n) => n.id === selectedRef.current) ??
          nodesRef.current[0];
        if (!target) {
          setError("No server available. Add a node first.");
          return;
        }
        await connect(target.id);
      }
      await refreshStatus();
    } catch (e) {
      fail(e);
    }
  };

  const handleConnectButton = async () => {
    setError(null);
    try {
      const target =
        nodesRef.current.find((n) => n.id === selectedRef.current) ??
        nodesRef.current[0];
      if (!target) {
        setError("No server available. Add a node first.");
        return;
      }
      await connect(target.id);
      await refreshStatus();
    } catch (e) {
      fail(e);
    }
  };

  const handleDisconnect = async () => {
    setError(null);
    try {
      await disconnect();
      await refreshStatus();
    } catch (e) {
      fail(e);
    }
  };

  const handleConnectNode = async (node: Node) => {
    setError(null);
    setSelectedId(node.id);
    try {
      await connect(node.id);
      await refreshStatus();
    } catch (e) {
      fail(e);
    }
  };

  const handleDeleteNode = async (node: Node) => {
    if (!window.confirm(`Delete node “${node.name}”?`)) return;
    setError(null);
    try {
      await deleteNode(node.id);
      setNodes((prev) => prev.filter((n) => n.id !== node.id));
      if (selectedId === node.id) setSelectedId(null);
      await refreshStatus();
    } catch (e) {
      fail(e);
    }
  };

  const handleCheckPingAll = async () => {
    setCheckingPing(true);
    setError(null);
    try {
      const results = await checkPingAll();
      setNodes((prev) =>
        prev.map((n) => {
          const r = results.find((x) => x.node_id === n.id);
          return r ? { ...n, ping_ms: r.ping_ms, loss: r.loss } : n;
        }),
      );
    } catch (e) {
      fail(e);
    } finally {
      setCheckingPing(false);
    }
  };

  const handleCheckPingOne = async (node: Node) => {
    setCheckingId(node.id);
    setError(null);
    try {
      const r = await checkPing(node.id);
      setNodes((prev) =>
        prev.map((n) =>
          n.id === node.id ? { ...n, ping_ms: r.ping_ms, loss: r.loss } : n,
        ),
      );
    } catch (e) {
      fail(e);
    } finally {
      setCheckingId(null);
    }
  };

  const handleUpdateSubscriptions = async () => {
    setUpdatingSubs(true);
    setError(null);
    try {
      const updated = await updateSubscriptions();
      setNodes((prev) => mergeNodes(prev, updated));
    } catch (e) {
      fail(e);
    } finally {
      setUpdatingSubs(false);
    }
  };

  const handleToggleSystemProxy = async (enabled: boolean) => {
    setError(null);
    try {
      await setSystemProxy(enabled);
      await refreshStatus();
    } catch (e) {
      fail(e);
    }
  };

  const handleToggleTun = async (enabled: boolean) => {
    setError(null);
    try {
      await setTun(enabled);
      await refreshStatus();
    } catch (e) {
      fail(e);
    }
  };

  const handleNodesChanged = (incoming: Node[]) => {
    setNodes((prev) => mergeNodes(prev, incoming));
  };

  const handleSettingsSaved = (s: Settings) => {
    setSettings(s);
    void refreshStatus();
  };

  return (
    <div className="flex h-full flex-col bg-adwaita-bg text-adwaita-text">
      <div className="flex items-center justify-between border-b border-adwaita-border bg-[#333333] px-4 py-2">
        <div className="flex items-center gap-2">
          <span className="flex h-7 w-7 items-center justify-center rounded-lg bg-adwaita-accent text-sm font-bold text-white">
            Z
          </span>
          <span className="text-sm font-semibold">Zavro Client</span>
        </div>
        <span className="text-xs text-adwaita-muted">
          {status?.connected ? "Connected" : "Disconnected"}
          {status?.core_running ? " · Xray running" : " · Xray stopped"}
        </span>
      </div>

      <TopBar
        status={status}
        checkingPing={checkingPing}
        updatingSubs={updatingSubs}
        onToggleConnected={() => void handleToggleConnected()}
        onConnect={() => void handleConnectButton()}
        onDisconnect={() => void handleDisconnect()}
        onCheckPing={() => void handleCheckPingAll()}
        onUpdateSubscriptions={() => void handleUpdateSubscriptions()}
        onOpenAddNode={(tab) => setAddNodeTab(tab)}
        onOpenSettings={() => setSettingsOpen(true)}
        onOpenLogs={() => setLogsOpen(true)}
      />

      <div className="flex min-h-0 flex-1">
        <Sidebar page={page} onNavigate={setPage} status={status} />

        <main className="flex min-w-0 flex-1 flex-col overflow-hidden">
          <div className="flex-1 overflow-y-auto p-4">
            {error && (
              <div className="mb-3 flex items-start justify-between gap-3 rounded-lg bg-adwaita-red/15 px-3 py-2 text-sm text-[#ff8a8a]">
                <span>{error}</span>
                <button
                  type="button"
                  onClick={() => setError(null)}
                  aria-label="Dismiss error"
                  className="shrink-0 rounded px-1.5 hover:bg-adwaita-panel"
                >
                  ✕
                </button>
              </div>
            )}

            <h1 className="mb-3 text-lg font-semibold">
              {PAGE_TITLES[page]}
            </h1>

            {page === "nodes" && (
              <NodesPage
                nodes={nodes}
                status={status}
                selectedId={selectedId}
                checkingId={checkingId}
                onSelect={(n) => setSelectedId(n.id)}
                onConnect={(n) => void handleConnectNode(n)}
                onDelete={(n) => void handleDeleteNode(n)}
                onCheckPing={(n) => void handleCheckPingOne(n)}
                onToggleSystemProxy={(v) => void handleToggleSystemProxy(v)}
                onToggleTun={(v) => void handleToggleTun(v)}
              />
            )}
            {page === "routing" && <RoutingPage />}
            {page === "subscriptions" && (
              <SubscriptionsPage onNodesChanged={handleNodesChanged} />
            )}
            {page === "proxy" && <ProxySettingsPage />}
            {page === "logs" && <LogsPage />}
            {page === "about" && <AboutPage status={status} />}
          </div>

          <div className="space-y-0 border-t border-adwaita-border px-4 pb-2 pt-3">
            <TrafficChart
              points={traffic?.points ?? []}
              dlBps={traffic?.dl_bps ?? status?.dl_bps ?? 0}
              ulBps={traffic?.ul_bps ?? status?.ul_bps ?? 0}
            />
          </div>
          <StatusBar status={status} coreInfo={coreInfo} settings={settings} />
        </main>
      </div>

      {addNodeTab && (
        <AddNodeModal
          initialTab={addNodeTab}
          onClose={() => setAddNodeTab(null)}
          onNodesChanged={handleNodesChanged}
        />
      )}
      {settingsOpen && (
        <SettingsModal
          onClose={() => setSettingsOpen(false)}
          onSaved={handleSettingsSaved}
        />
      )}
      {logsOpen && <LogsModal onClose={() => setLogsOpen(false)} />}
    </div>
  );
}
