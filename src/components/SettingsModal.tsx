import { useEffect, useState } from "react";
import { X } from "lucide-react";
import type { Settings } from "../types";
import { getSettings, saveSettings } from "../api";

interface Props {
  onClose: () => void;
  onSaved: (s: Settings) => void;
}

export default function SettingsModal({ onClose, onSaved }: Props) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);

  useEffect(() => {
    let alive = true;
    getSettings()
      .then((s) => {
        if (alive) setSettings(s);
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

  const set = <K extends keyof Settings>(key: K, value: Settings[K]) => {
    setSettings((prev) => (prev ? { ...prev, [key]: value } : prev));
    setDone(false);
  };

  const handleSave = async () => {
    if (!settings) return;
    if (
      !Number.isInteger(settings.socks_port) ||
      settings.socks_port < 1 ||
      settings.socks_port > 65535 ||
      !Number.isInteger(settings.http_port) ||
      settings.http_port < 1 ||
      settings.http_port > 65535
    ) {
      setError("Ports must be numbers between 1 and 65535.");
      return;
    }
    setSaving(true);
    setError(null);
    try {
      await saveSettings(settings);
      onSaved(settings);
      setDone(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  };

  const inputCls =
    "w-full rounded-lg border border-adwaita-border bg-adwaita-bg px-3 py-2 text-sm text-adwaita-text";

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
        aria-label="Settings"
      >
        <div className="flex items-center justify-between border-b border-adwaita-border px-5 py-4">
          <h2 className="text-base font-semibold">Settings</h2>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close"
            className="rounded-md p-1 text-adwaita-muted hover:bg-adwaita-panel hover:text-adwaita-text"
          >
            <X size={18} />
          </button>
        </div>

        <div className="space-y-4 px-5 py-4">
          {loading && (
            <p className="text-sm text-adwaita-muted">Loading settings…</p>
          )}
          {error && (
            <p className="rounded-lg bg-adwaita-red/15 px-3 py-2 text-sm text-[#ff8a8a]">
              {error}
            </p>
          )}
          {settings && (
            <>
              <div className="grid grid-cols-2 gap-3">
                <label className="block text-sm text-adwaita-muted">
                  SOCKS5 port
                  <input
                    type="number"
                    min={1}
                    max={65535}
                    value={settings.socks_port}
                    onChange={(e) =>
                      set("socks_port", Number(e.target.value))
                    }
                    className={`${inputCls} mt-1`}
                  />
                </label>
                <label className="block text-sm text-adwaita-muted">
                  HTTP port
                  <input
                    type="number"
                    min={1}
                    max={65535}
                    value={settings.http_port}
                    onChange={(e) => set("http_port", Number(e.target.value))}
                    className={`${inputCls} mt-1`}
                  />
                </label>
                <label className="block text-sm text-adwaita-muted">
                  Routing mode
                  <select
                    value={settings.routing_mode}
                    onChange={(e) => set("routing_mode", e.target.value)}
                    className={`${inputCls} mt-1`}
                  >
                    <option value="rule">Rule (PAC)</option>
                    <option value="global">Global</option>
                    <option value="direct">Direct</option>
                  </select>
                </label>
                <label className="block text-sm text-adwaita-muted">
                  Auto-update subscriptions (hours)
                  <input
                    type="number"
                    min={0}
                    value={settings.auto_update_hours}
                    onChange={(e) =>
                      set("auto_update_hours", Number(e.target.value))
                    }
                    className={`${inputCls} mt-1`}
                  />
                </label>
                <label className="col-span-2 block text-sm text-adwaita-muted">
                  DNS servers (comma separated)
                  <input
                    value={settings.dns.join(", ")}
                    onChange={(e) =>
                      set(
                        "dns",
                        e.target.value
                          .split(",")
                          .map((d) => d.trim())
                          .filter(Boolean),
                      )
                    }
                    placeholder="1.1.1.1, 8.8.8.8"
                    className={`${inputCls} mt-1`}
                  />
                </label>
                <label className="block text-sm text-adwaita-muted">
                  Log level
                  <select
                    value={settings.log_level}
                    onChange={(e) => set("log_level", e.target.value)}
                    className={`${inputCls} mt-1`}
                  >
                    <option value="debug">debug</option>
                    <option value="info">info</option>
                    <option value="warn">warn</option>
                    <option value="error">error</option>
                  </select>
                </label>
              </div>

              <label className="flex items-center justify-between rounded-lg bg-adwaita-panel px-3 py-2.5 text-sm">
                System Proxy
                <input
                  type="checkbox"
                  checked={settings.system_proxy}
                  onChange={(e) => set("system_proxy", e.target.checked)}
                  className="h-4 w-4 accent-[#3584e4]"
                />
              </label>
              <label className="flex items-center justify-between rounded-lg bg-adwaita-panel px-3 py-2.5 text-sm">
                TUN Mode
                <input
                  type="checkbox"
                  checked={settings.tun_mode}
                  onChange={(e) => set("tun_mode", e.target.checked)}
                  className="h-4 w-4 accent-[#3584e4]"
                />
              </label>

              <button
                type="button"
                onClick={handleSave}
                disabled={saving}
                className="w-full rounded-lg bg-adwaita-accent px-4 py-2.5 text-sm font-medium text-white hover:bg-adwaita-accentHover disabled:opacity-50"
              >
                {saving ? "Saving…" : "Save Settings"}
              </button>
              {done && (
                <p className="rounded-lg bg-adwaita-green/15 px-3 py-2 text-sm text-adwaita-green">
                  Settings saved.
                </p>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
