import { useState } from "react";
import { X } from "lucide-react";
import type { AddNodeTab } from "./TopBar";
import type { Node, Protocol } from "../types";
import { addNode, addSubscription, importLink, importSubscription } from "../api";

interface Props {
  initialTab: AddNodeTab;
  onClose: () => void;
  onNodesChanged: (nodes: Node[]) => void;
}

const PROTOCOLS: Protocol[] = ["vless", "vmess", "trojan", "shadowsocks"];

function newId(): string {
  return `node-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

export default function AddNodeModal({ initialTab, onClose, onNodesChanged }: Props) {
  const [tab, setTab] = useState<AddNodeTab>(initialTab);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState<string | null>(null);

  const [link, setLink] = useState("");
  const [subUrl, setSubUrl] = useState("");

  const [name, setName] = useState("");
  const [address, setAddress] = useState("");
  const [port, setPort] = useState("443");
  const [protocol, setProtocol] = useState<Protocol>("vless");
  const [transport, setTransport] = useState("tcp");
  const [security, setSecurity] = useState("tls");
  const [group, setGroup] = useState("");
  const [rawLink, setRawLink] = useState("");

  const fail = (e: unknown) => {
    setError(e instanceof Error ? e.message : String(e));
  };

  const handleImportLink = async () => {
    const value = link.trim();
    if (!value) {
      setError("Paste a node link first.");
      return;
    }
    setBusy(true);
    setError(null);
    setDone(null);
    try {
      const node = await importLink(value);
      onNodesChanged([node]);
      setDone(`Imported “${node.name}”.`);
      setLink("");
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  };

  const handleManual = async () => {
    if (!name.trim() || !address.trim()) {
      setError("Name and address are required.");
      return;
    }
    const portNum = Number(port);
    if (!Number.isInteger(portNum) || portNum < 1 || portNum > 65535) {
      setError("Port must be a number between 1 and 65535.");
      return;
    }
    setBusy(true);
    setError(null);
    setDone(null);
    try {
      const node: Node = {
        id: newId(),
        name: name.trim(),
        address: address.trim(),
        port: portNum,
        protocol,
        transport: transport.trim() || undefined,
        security: security.trim() || undefined,
        group: group.trim() || undefined,
        raw_link: rawLink.trim() || undefined,
      };
      const created = await addNode(node);
      onNodesChanged([created]);
      setDone(`Added “${created.name}”.`);
      setName("");
      setAddress("");
      setPort("443");
      setRawLink("");
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  };

  const handleSubscription = async () => {
    const value = subUrl.trim();
    if (!value) {
      setError("Paste a subscription URL first.");
      return;
    }
    setBusy(true);
    setError(null);
    setDone(null);
    try {
      await addSubscription(value);
      const nodes = await importSubscription(value);
      onNodesChanged(nodes);
      setDone(`Subscription added — ${nodes.length} node(s) imported.`);
      setSubUrl("");
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  };

  const tabBtn = (id: AddNodeTab, label: string) => (
    <button
      key={id}
      type="button"
      onClick={() => {
        setTab(id);
        setError(null);
        setDone(null);
      }}
      className={`rounded-lg px-3 py-2 text-sm ${
        tab === id
          ? "bg-adwaita-accent text-white"
          : "bg-adwaita-panel text-adwaita-muted hover:text-adwaita-text"
      }`}
    >
      {label}
    </button>
  );

  const inputCls =
    "w-full rounded-lg border border-adwaita-border bg-adwaita-bg px-3 py-2 text-sm text-adwaita-text placeholder:text-adwaita-muted";

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-lg rounded-2xl border border-adwaita-border bg-adwaita-bg shadow-2xl"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-label="Add node"
      >
        <div className="flex items-center justify-between border-b border-adwaita-border px-5 py-4">
          <h2 className="text-base font-semibold">Add Node</h2>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close"
            className="rounded-md p-1 text-adwaita-muted hover:bg-adwaita-panel hover:text-adwaita-text"
          >
            <X size={18} />
          </button>
        </div>

        <div className="flex gap-2 px-5 pt-4">
          {tabBtn("link", "Paste Link")}
          {tabBtn("manual", "Manual")}
          {tabBtn("subscription", "Subscription URL")}
        </div>

        <div className="space-y-3 px-5 py-4">
          {tab === "link" && (
            <>
              <label className="block text-sm text-adwaita-muted">
                Node link (VMess, VLESS, Trojan or Shadowsocks)
                <textarea
                  value={link}
                  onChange={(e) => setLink(e.target.value)}
                  rows={4}
                  placeholder="vless://… / vmess://… / trojan://… / ss://…"
                  className={`${inputCls} mt-1 resize-none font-mono text-xs`}
                />
              </label>
              <button
                type="button"
                onClick={handleImportLink}
                disabled={busy}
                className="w-full rounded-lg bg-adwaita-accent px-4 py-2.5 text-sm font-medium text-white hover:bg-adwaita-accentHover disabled:opacity-50"
              >
                {busy ? "Importing…" : "Import Node"}
              </button>
            </>
          )}

          {tab === "manual" && (
            <>
              <div className="grid grid-cols-2 gap-3">
                <label className="block text-sm text-adwaita-muted">
                  Name *
                  <input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="DE - Frankfurt 01"
                    className={`${inputCls} mt-1`}
                  />
                </label>
                <label className="block text-sm text-adwaita-muted">
                  Protocol
                  <select
                    value={protocol}
                    onChange={(e) => setProtocol(e.target.value as Protocol)}
                    className={`${inputCls} mt-1`}
                  >
                    {PROTOCOLS.map((p) => (
                      <option key={p} value={p}>
                        {p.toUpperCase()}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="block text-sm text-adwaita-muted">
                  Address *
                  <input
                    value={address}
                    onChange={(e) => setAddress(e.target.value)}
                    placeholder="de-01.zavro.net"
                    className={`${inputCls} mt-1`}
                  />
                </label>
                <label className="block text-sm text-adwaita-muted">
                  Port *
                  <input
                    value={port}
                    onChange={(e) => setPort(e.target.value)}
                    inputMode="numeric"
                    className={`${inputCls} mt-1`}
                  />
                </label>
                <label className="block text-sm text-adwaita-muted">
                  Transport
                  <input
                    value={transport}
                    onChange={(e) => setTransport(e.target.value)}
                    placeholder="tcp / ws / grpc"
                    className={`${inputCls} mt-1`}
                  />
                </label>
                <label className="block text-sm text-adwaita-muted">
                  Security
                  <input
                    value={security}
                    onChange={(e) => setSecurity(e.target.value)}
                    placeholder="tls / reality / none"
                    className={`${inputCls} mt-1`}
                  />
                </label>
                <label className="col-span-2 block text-sm text-adwaita-muted">
                  Group
                  <input
                    value={group}
                    onChange={(e) => setGroup(e.target.value)}
                    placeholder="Zavro"
                    className={`${inputCls} mt-1`}
                  />
                </label>
                <label className="col-span-2 block text-sm text-adwaita-muted">
                  Raw link (optional)
                  <input
                    value={rawLink}
                    onChange={(e) => setRawLink(e.target.value)}
                    placeholder="vless://…"
                    className={`${inputCls} mt-1 font-mono text-xs`}
                  />
                </label>
              </div>
              <button
                type="button"
                onClick={handleManual}
                disabled={busy}
                className="w-full rounded-lg bg-adwaita-accent px-4 py-2.5 text-sm font-medium text-white hover:bg-adwaita-accentHover disabled:opacity-50"
              >
                {busy ? "Adding…" : "Add Node"}
              </button>
            </>
          )}

          {tab === "subscription" && (
            <>
              <label className="block text-sm text-adwaita-muted">
                Subscription URL
                <input
                  value={subUrl}
                  onChange={(e) => setSubUrl(e.target.value)}
                  placeholder="https://example.com/sub/…"
                  className={`${inputCls} mt-1 font-mono text-xs`}
                />
              </label>
              <p className="text-xs text-adwaita-muted">
                The URL is saved as a subscription and its nodes are imported.
                Use Update Subscriptions later to refresh it.
              </p>
              <button
                type="button"
                onClick={handleSubscription}
                disabled={busy}
                className="w-full rounded-lg bg-adwaita-accent px-4 py-2.5 text-sm font-medium text-white hover:bg-adwaita-accentHover disabled:opacity-50"
              >
                {busy ? "Importing…" : "Add Subscription & Import Nodes"}
              </button>
            </>
          )}

          {error && (
            <p className="rounded-lg bg-adwaita-red/15 px-3 py-2 text-sm text-[#ff8a8a]">
              {error}
            </p>
          )}
          {done && (
            <p className="rounded-lg bg-adwaita-green/15 px-3 py-2 text-sm text-adwaita-green">
              {done}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
