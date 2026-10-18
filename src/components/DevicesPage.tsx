import {
  CheckIcon,
  FileWarningIcon,
  FlaskConicalIcon,
  LoaderIcon,
  PauseIcon,
  PlayIcon,
  RefreshCwIcon,
  TriangleAlertIcon,
  WifiIcon,
} from "lucide-react";
import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Device, Transfer } from "../types";
import { DeviceCard } from "../components/DeviceCard";
import { formatBytes } from "../format";
import { SendFileModal } from "./SendFileModel";
import { useDevices } from "../hooks/useDevices";

interface DevicesPageProps {
  recentTransfers: Transfer[];
  onSendFiles: (device: Device, path: string) => void;
  onSelfTest: (path: string) => void;
  resumableIds: Set<string>;
  onResume: (transferId: string) => void;
  selfTestRunning?: boolean;
}

const STATUS_META: Record<
  string,
  { icon: React.ReactNode; className: string; label: string }
> = {
  completed: { icon: <CheckIcon size={14} />, className: "text-success", label: "Complete" },
  paused: { icon: <PauseIcon size={14} />, className: "text-warning", label: "Paused" },
  failed: { icon: <FileWarningIcon size={14} />, className: "text-danger", label: "Failed" },
  transferring: { icon: <div className="w-2 h-2 rounded-full bg-accent-light animate-pulse" />, className: "text-accent-light", label: "In progress" },
  connecting: { icon: <LoaderIcon size={14} className="animate-spin" />, className: "text-accent-light", label: "Connecting" },
};

export function DevicesPage({
  recentTransfers,
  onSendFiles,
  onSelfTest,
  resumableIds,
  onResume,
  selfTestRunning,
}: DevicesPageProps) {
  const { devices, loading, error, onlineCount, scan } = useDevices();
  const [sendTarget, setSendTarget] = useState<Device | null>(null);
  const [pickingTest, setPickingTest] = useState(false);

  const pickAndSelfTest = async () => {
    try {
      setPickingTest(true);
      const selected = await open({
        multiple: false,
        directory: false,
        title: "Select a file to test transfer",
      });
      if (!selected) return;
      const path = Array.isArray(selected) ? selected[0] : selected;
      onSelfTest(path);
    } catch (e) {
      console.log("Failed to pick file for self-test::", e);
    } finally {
      setPickingTest(false);
    }
  };

  return (
    <div className="page">
      {/* Page Header */}
      <header className="page-header">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <span
              className={`status-dot inline-block ${
                error ? "status-dot-error" : onlineCount > 0 ? "" : "status-dot-off"
              }`}
            />
            <span className="text-xs font-medium text-text-muted">
              {onlineCount} device{onlineCount === 1 ? "" : "s"} online
            </span>
          </div>
          <h1 className="page-title">Devices Nearby</h1>
          <p className="page-subtitle">
            {onlineCount} device{onlineCount === 1 ? "" : "s"} available for transfer on this network
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            className="btn btn-secondary"
            onClick={pickAndSelfTest}
            disabled={selfTestRunning || pickingTest}
            title="Send a file to this device to verify file transfer works"
          >
            {selfTestRunning || pickingTest ? (
              <LoaderIcon size={16} className="animate-spin" />
            ) : (
              <FlaskConicalIcon size={16} />
            )}
            {selfTestRunning ? "Testing..." : "Test on this device"}
          </button>
          <button
            className="btn btn-secondary"
            onClick={scan}
            disabled={loading}
          >
            {loading ? (
              <LoaderIcon size={16} className="animate-spin" />
            ) : (
              <WifiIcon size={16} />
            )}
            {loading ? "Scanning..." : "Scan"}
          </button>
        </div>
      </header>

      {/* Device Grid */}
      {loading ? (
        <div className="flex items-center justify-center py-12">
          <LoaderIcon size={32} className="animate-spin text-accent-light" />
          <span className="ml-3 text-text-muted">Scanning for devices...</span>
        </div>
      ) : error ? (
        <div className="empty-state mb-6">
          <TriangleAlertIcon size={32} className="mx-auto mb-2 text-danger" />
          <p>{error}</p>
          <div className="mt-3 flex justify-center">
            <button
              className="btn btn-secondary text-sm"
              onClick={scan}
            >
              <RefreshCwIcon size={14} />
              Try again
            </button>
          </div>
        </div>
      ) : devices.length > 0 ? (
        <div className="grid grid-cols-2 gap-4 mb-6">
          {devices.map((device) => (
            <DeviceCard key={device.id} device={device} onSend={setSendTarget} />
          ))}
        </div>
      ) : (
        <div className="empty-state mb-6">
          <WifiIcon size={32} className="mx-auto mb-2 text-text-faint" />
          <p>No devices found on your network</p>
          <p className="text-xs text-text-faint mt-1">
            Make sure you're connected to a network and try scanning again.
          </p>
        </div>
      )}

      {/* Recent Transfers */}
      {recentTransfers.length > 0 && (
        <section className="card">
          <div className="card-header">
            <h2 className="card-title">Recent Transfers</h2>
            <span className="badge badge-accent">
              {recentTransfers.filter((t) => t.status === "completed").length} complete
            </span>
          </div>

          <div className="space-y-2">
            {recentTransfers.map((t) => {
              const meta = STATUS_META[t.status] ?? STATUS_META.completed;
              const progress = t.sizeBytes > 0 ? (t.transferredBytes / t.sizeBytes) * 100 : 0;
              const canResume = resumableIds.has(t.id);

              return (
                <div
                  key={t.id}
                  className="flex flex-col sm:flex-row sm:items-center gap-2 p-3 rounded-lg bg-bg-card-hover border border-border"
                >
                  <div className="flex items-center gap-3 flex-1 min-w-0">
                    <div className={`${meta.className} flex-shrink-0`}>{meta.icon}</div>

                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium truncate">{t.filename}</p>
                      <p className="text-xs text-text-muted">
                        {t.direction === "sent" ? "To" : "From"} {t.deviceName}
                      </p>
                    </div>
                  </div>

                  {t.status === "transferring" && (
                    <div className="flex items-center gap-3 w-full sm:w-auto">
                      <div className="flex-1 sm:w-32 h-1.5 rounded-full bg-bg-card-hover overflow-hidden">
                        <div
                          className="h-full rounded-full bg-accent-gradient transition-all duration-300"
                          style={{ width: `${Math.min(progress, 100)}%` }}
                        />
                      </div>
                      <span className="text-xs text-text-muted font-mono whitespace-nowrap">
                        {formatBytes(t.speedMBps * 1024 * 1024)}/s
                      </span>
                    </div>
                  )}

                  <div className="flex items-center gap-3 flex-shrink-0">
                    <span className={`text-xs font-medium ${meta.className}`}>{meta.label}</span>
                    <span className="text-xs text-text-muted font-mono">
                      {formatBytes(t.sizeBytes)}
                    </span>
                    {canResume && (
                      <button
                        className="btn btn-secondary text-xs px-2 py-1 inline-flex items-center gap-1"
                        onClick={() => onResume(t.id)}
                      >
                        <PlayIcon size={12} />
                        Resume
                      </button>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </section>
      )}

      {/* Send File Modal */}
      {sendTarget && (
        <SendFileModal
          device={sendTarget}
          onClose={() => setSendTarget(null)}
          onSend={(path) => {
            onSendFiles(sendTarget,path);
            setSendTarget(null);
          }}
        />
      )}
    </div>
  );
}
