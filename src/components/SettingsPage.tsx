import { useEffect, useState } from "react";
import type { Device } from "../types";
import { ShieldIcon, CheckCircleIcon, FolderIcon, InfoIcon } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useTheme } from "../context/ThemeContext";
import { downloadDir } from "@tauri-apps/api/path";

interface SettingsPageProps {
  localDeviceName: string;
  localIp: string;
  trustedDevices: Device[];
}

const themeOptions = [
  { value: "system", label: "System" },
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
] as const;

export function SettingsPage({ localDeviceName, localIp }: SettingsPageProps) {
  const [identity, setIdentity] = useState<{ id: string; name: string; port: number } | null>(null);
  const [isDiscoveryEnabled, setIsDiscoveryEnabled] = useState(true);
  const [isDiscoveryLoading, setIsDiscoveryLoading] = useState(true);
  const [isThemeDropdownOpen, setIsThemeDropdownOpen] = useState(false);
  const { theme, setTheme } = useTheme();
  const [downloadPath, setDownloadPath] = useState<string | null>(null);

  useEffect(() => {
    const load = async () => {
      try {
        const info = await invoke<{ id: string; name: string; port: number }>("get_identity");
        setIdentity(info);
      } catch (err) {
        console.log("Failed to get identity::", err);
      }
    };
    load();
  }, []);

  useEffect(() => {
    const loadDiscoveryState = async () => {
      try {
        const state = await invoke<boolean>("get_discovery_state");
        setIsDiscoveryEnabled(state);
      } catch (error) {
        console.log("Failed to load discovery state:", error);
      } finally {
        setIsDiscoveryLoading(false);
      }
    };
    loadDiscoveryState();
  }, []);

  useEffect(() => {
    downloadDir()
      .then(setDownloadPath)
      .catch((e) => console.log("Failed to resolve downloads dir::", e));
  }, []);

  const toggleDiscovery = async () => {
    const newState = !isDiscoveryEnabled;
    try {
      await invoke("set_discovery_state", { enabled: newState });
      setIsDiscoveryEnabled(newState);
    } catch (error) {
      console.log("Failed to toggle discovery", error);
    }
  };

  const currentTheme = themeOptions.find((t) => t.value === theme);

  return (
    <div className="page">
      {/* Page Header */}
      <header className="page-header">
        <div>
          <h1 className="page-title">Settings</h1>
          <p className="page-subtitle">Device identity, trusted peers, and security</p>
        </div>
      </header>

      {/* This device */}
      <section className="card mb-4">
        <h2 className="card-title">This device</h2>
        <div className="space-y-3">
          <div className="flex items-center justify-between py-2 border-b border-border">
            <span className="text-sm text-text-muted">Device name</span>
            <span className="text-sm font-medium">{identity?.name ?? localDeviceName}</span>
          </div>
          <div className="flex items-center justify-between py-2 border-b border-border">
            <span className="text-sm text-text-muted">Local IP</span>
            <span className="text-sm font-mono text-text">{localIp}</span>
          </div>
          <div className="flex items-center justify-between py-2 border-b border-border">
            <span className="text-sm text-text-muted">Discovery status</span>
            <span className="flex items-center gap-2">
              <span
                className={`status-dot inline-block ${isDiscoveryEnabled ? "" : "status-dot-off"}`}
              />
              <span
                className={`text-sm ${isDiscoveryEnabled ? "text-success" : "text-text-muted"}`}
              >
                {isDiscoveryEnabled ? "Discoverable" : "Hidden"}
              </span>
              <button
                type="button"
                onClick={toggleDiscovery}
                disabled={isDiscoveryLoading}
                className={`relative w-10 h-5 rounded-full transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed ${
                  isDiscoveryEnabled ? "bg-success" : "bg-danger-dim"
                }`}
                role="switch"
                aria-checked={isDiscoveryEnabled}
                aria-label="Toggle discovery"
              >
                <span
                  className={`absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
                    isDiscoveryEnabled ? "translate-x-5" : "translate-x-0"
                  }`}
                />
              </button>
            </span>
          </div>
          <div className="flex items-center justify-between py-2">
            <span className="text-sm text-text-muted">Theme</span>
            <div className="relative">
              <button
                type="button"
                onClick={() => setIsThemeDropdownOpen(!isThemeDropdownOpen)}
                className="flex items-center gap-2 px-3 py-1.5 rounded-md bg-bg-card-hover border border-border hover:border-border-strong transition-colors min-w-[120px] cursor-pointer"
              >
                {currentTheme && (
                  <span className="text-sm flex-1 text-center">{currentTheme.label}</span>
                )}
              </button>
              {isThemeDropdownOpen && (
                <div className="absolute right-0 top-full mt-1 w-full min-w-[140px] bg-bg-card border border-border rounded-md shadow-lg overflow-hidden z-10">
                  {themeOptions.map((option) => {
                    const isActive = theme === option.value;
                    return (
                      <button
                        key={option.value}
                        type="button"
                        onClick={() => {
                          setTheme(option.value);
                          setIsThemeDropdownOpen(false);
                        }}
                        className={`w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors cursor-pointer ${
                          isActive
                            ? "bg-accent-gradient text-text"
                            : "text-text hover:bg-bg-card-hover"
                        }`}
                      >
                        {option.label}
                      </button>
                    );
                  })}
                </div>
              )}
            </div>
          </div>
        </div>
      </section>

      {/* Received files */}
      <section className="card mb-4">
        <div className="card-header">
          <h2 className="card-title">Received files</h2>
          <span className="badge badge-accent">Default</span>
        </div>

        <div className="flex items-center gap-3 p-2.5 rounded bg-bg-card-hover border border-border">
          <FolderIcon size={16} className="text-accent-light shrink-0" />
          <div className="min-w-0 flex-1">
            <p className="text-xs text-text-muted">Saved to</p>
            <p
              className="text-sm font-mono text-text truncate"
              title={downloadPath ?? undefined}
            >
              {downloadPath ?? "Resolving…"}
            </p>
          </div>
          <button
            type="button"
            className="btn btn-secondary text-xs"
            disabled
            title="Custom save location is not available yet"
          >
            Change
          </button>
        </div>

        <p className="flex items-start gap-1.5 text-xs text-text-faint mt-3">
          <InfoIcon size={13} className="shrink-0 mt-px" />
          Incoming files are saved to your Downloads folder. Choosing a custom location is coming
          soon.
        </p>
      </section>

      {/* Trusted devices
      <section className="card mb-4">
        <div className="card-header">
          <h2 className="card-title">Trusted devices</h2>
          <span className="badge badge-accent">{trustedDevices.length} paired</span>
        </div>

        {trustedDevices.length > 0 ? (
          <div className="space-y-2">
            {trustedDevices.map((device) => (
              <div
                key={device.id}
                className="flex items-center justify-between p-2.5 rounded bg-bg-card-hover border border-border"
              >
                <div>
                  <p className="text-sm font-medium">{device.name}</p>
                  <p className="text-xs text-text-muted font-mono">{device.ip}</p>
                </div>
                <span className="w-fit badge badge-success flex flex-row items-center gap-1">
                  <LockIcon size={12} /> PAIRED
                </span>
              </div>
            ))}
          </div>
        ) : (
          <div className="empty-state text-sm">
            <p>No trusted devices yet</p>
            <p className="text-xs text-text-faint mt-1">
              Pair with devices to enable secure transfers
            </p>
          </div>
        )}
      </section> */}

      {/* Security */}
      <section className="card">
        <h2 className="card-title">Security</h2>
        <div className="space-y-3">
          <div className="flex items-center justify-between py-2 border-b border-border">
            <span className="text-sm text-text-muted">Transport encryption</span>
            <span className="flex items-center gap-1.5 text-sm text-success">
              <CheckCircleIcon size={14} />
              TLS enabled
            </span>
          </div>
          <div className="flex items-center justify-between py-2">
            <span className="text-sm text-text-muted">Integrity check</span>
            <span className="flex items-center gap-1.5 text-sm text-text">
              <ShieldIcon size={14} className="text-accent-light" />
              SHA-256 on every completed transfer
            </span>
          </div>
        </div>
      </section>
    </div>
  );
}