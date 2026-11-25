import { useEffect, useState } from "react";
import { Plus, Trash2 } from "lucide-react";
import type { RoutingRules } from "../types";
import { getRoutingRules, saveRoutingRules, setRoutingMode } from "../api";

const MODES = [
  { value: "rule", label: "Rule (PAC)", desc: "Route by rules; bypassed domains and IPs go direct." },
  { value: "global", label: "Global", desc: "Route all traffic through the proxy." },
  { value: "direct", label: "Direct", desc: "Bypass the proxy for all traffic." },
];

export default function RoutingPage() {
  const [rules, setRules] = useState<RoutingRules | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);
  const [newDomain, setNewDomain] = useState("");
  const [newIp, setNewIp] = useState("");

  useEffect(() => {
    let alive = true;
    getRoutingRules()
      .then((r) => {
        if (alive) setRules(r);
      })
      .catch((e: unknown) => {
        if (alive) setError(e instanceof Error ? e.message : String(e));
      })
      .finally(() => {
        if (alive) setLoading(false);
      });
    return () => {
      alive = false;
    };
  }, []);

  const persist = async (next: RoutingRules) => {
    setSaving(true);
    setError(null);
    setDone(false);
    try {
      await saveRoutingRules(next);
      await setRoutingMode(next.mode);
      setRules(next);
      setDone(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  };

  const changeMode = (mode: string) => {
    if (!rules) return;
    void persist({ ...rules, mode });
  };

  const addDomain = () => {
    if (!rules) return;
    const value = newDomain.trim().toLowerCase();
    if (!value || rules.bypass_domains.includes(value)) return;
    setNewDomain("");
    void persist({ ...rules, bypass_domains: [...rules.bypass_domains, value] });
  };

  const removeDomain = (domain: string) => {
    if (!rules) return;
    void persist({
      ...rules,
      bypass_domains: rules.bypass_domains.filter((d) => d !== domain),
    });
  };

  const addIp = () => {
    if (!rules) return;
    const value = newIp.trim();
    if (!value || rules.bypass_ips.includes(value)) return;
    setNewIp("");
    void persist({ ...rules, bypass_ips: [...rules.bypass_ips, value] });
  };

  const removeIp = (ip: string) => {
    if (!rules) return;
    void persist({
      ...rules,
      bypass_ips: rules.bypass_ips.filter((x) => x !== ip),
    });
  };

  const inputCls =
    "rounded-lg border border-adwaita-border bg-adwaita-panel px-3 py-2 text-sm text-adwaita-text placeholder:text-adwaita-muted";

  if (loading) return <p className="text-sm text-adwaita-muted">Loading routing rules…</p>;

  return (
    <div className="max-w-2xl space-y-6">
      <section>
        <h2 className="mb-3 text-base font-semibold">Routing Mode</h2>
        <div className="space-y-2">
          {MODES.map((m) => (
            <label
              key={m.value}
              className={`flex cursor-pointer items-start gap-3 rounded-xl border px-4 py-3 ${
                rules?.mode === m.value
                  ? "border-adwaita-accent bg-adwaita-panel"
                  : "border-adwaita-border bg-adwaita-panel/50 hover:bg-adwaita-panel"
              }`}
            >
              <input
                type="radio"
                name="routing-mode"
                value={m.value}
                checked={rules?.mode === m.value}
                onChange={() => changeMode(m.value)}
                disabled={saving}
                className="mt-1 accent-[#3584e4]"
              />
              <span>
                <span className="block text-sm font-medium">{m.label}</span>
                <span className="block text-xs text-adwaita-muted">{m.desc}</span>
              </span>
            </label>
          ))}
        </div>
      </section>

      <section>
        <h2 className="mb-3 text-base font-semibold">Bypass Domains</h2>
        <div className="flex gap-2">
          <input
            value={newDomain}
            onChange={(e) => setNewDomain(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") addDomain();
            }}
            placeholder="example.com"
            aria-label="New bypass domain"
            className={`${inputCls} flex-1`}
          />
          <button
            type="button"
            onClick={addDomain}
            disabled={saving || !newDomain.trim()}
            className="flex items-center gap-1 rounded-lg bg-adwaita-accent px-3 py-2 text-sm font-medium text-white hover:bg-adwaita-accentHover disabled:opacity-50"
          >
            <Plus size={15} /> Add
          </button>
        </div>
        <ul className="mt-3 space-y-1.5">
          {(rules?.bypass_domains ?? []).map((d) => (
            <li
              key={d}
              className="flex items-center justify-between rounded-lg bg-adwaita-panel px-3 py-2 text-sm"
            >
              {d}
              <button
                type="button"
                onClick={() => removeDomain(d)}
                aria-label={`Remove ${d}`}
                className="rounded p-1 text-adwaita-muted hover:text-adwaita-red"
              >
                <Trash2 size={15} />
              </button>
            </li>
          ))}
          {(rules?.bypass_domains ?? []).length === 0 && (
            <li className="text-sm text-adwaita-muted">No bypass domains.</li>
          )}
        </ul>
      </section>

      <section>
        <h2 className="mb-3 text-base font-semibold">Bypass IPs / CIDR</h2>
        <div className="flex gap-2">
          <input
            value={newIp}
            onChange={(e) => setNewIp(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") addIp();
            }}
            placeholder="192.168.0.0/16"
            aria-label="New bypass IP"
            className={`${inputCls} flex-1`}
          />
          <button
            type="button"
            onClick={addIp}
            disabled={saving || !newIp.trim()}
            className="flex items-center gap-1 rounded-lg bg-adwaita-accent px-3 py-2 text-sm font-medium text-white hover:bg-adwaita-accentHover disabled:opacity-50"
          >
            <Plus size={15} /> Add
          </button>
        </div>
        <ul className="mt-3 space-y-1.5">
          {(rules?.bypass_ips ?? []).map((ip) => (
            <li
              key={ip}
              className="flex items-center justify-between rounded-lg bg-adwaita-panel px-3 py-2 text-sm"
            >
              {ip}
              <button
                type="button"
                onClick={() => removeIp(ip)}
                aria-label={`Remove ${ip}`}
                className="rounded p-1 text-adwaita-muted hover:text-adwaita-red"
              >
                <Trash2 size={15} />
              </button>
            </li>
          ))}
          {(rules?.bypass_ips ?? []).length === 0 && (
            <li className="text-sm text-adwaita-muted">No bypass IPs.</li>
          )}
        </ul>
      </section>

      {error && (
        <p className="rounded-lg bg-adwaita-red/15 px-3 py-2 text-sm text-[#ff8a8a]">{error}</p>
      )}
      {done && (
        <p className="rounded-lg bg-adwaita-green/15 px-3 py-2 text-sm text-adwaita-green">
          Routing rules saved.
        </p>
      )}
    </div>
  );
}
